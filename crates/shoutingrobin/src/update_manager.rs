use crate::app_settings::AppSettings;
use anyhow::Context as _;
use gpui::{App, AppContext, Entity, Global};
use semver::Version;
use std::path::PathBuf;
use std::time::Duration;

const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60); // 1 hour
const GITHUB_OWNER: &str = "zanmato";
const GITHUB_REPO: &str = "shouting-robin";

// The macOS .app bundle ships the executable under this path (see the release
// workflow). The name contains a space because the bundle is user-facing.
const MACOS_BIN_IN_ARCHIVE: &str = "Shouting Robin.app/Contents/MacOS/Shouting Robin";

/// Release assets the workflow publishes next to the binaries: the SHA-256 of
/// every asset, and a minisign signature over that file.
const CHECKSUMS_ASSET: &str = "SHA256SUMS";
const CHECKSUMS_SIGNATURE_ASSET: &str = "SHA256SUMS.minisig";

/// The minisign public key releases are signed with. `None` until a key pair
/// exists (see `scripts/generate-update-key.sh`); with no key the updater
/// still verifies checksums but cannot tell a release of ours from one
/// published by whoever holds the GitHub account.
const UPDATE_PUBLIC_KEY: Option<&str> = None;

#[derive(Clone, Debug)]
pub struct ReleaseInfo {
    pub version: String,
    #[allow(dead_code)]
    pub html_url: String,
}

pub struct UpdateManager {
    pub pending_update: Entity<Option<ReleaseInfo>>,
    pub just_updated_from: Entity<Option<String>>,
}

impl Global for UpdateManager {}

fn sha256_of_file(path: &std::path::Path) -> anyhow::Result<String> {
    use sha2::Digest as _;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = sha2::Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}

/// Checks `path` against the `sha256sum`-format line for `asset_name`.
fn verify_sha256(path: &std::path::Path, asset_name: &str, checksums: &str) -> anyhow::Result<()> {
    let expected = checksums
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let digest = parts.next()?;
            let name = parts.next()?.trim_start_matches('*');
            (name == asset_name).then(|| digest.to_ascii_lowercase())
        })
        .next()
        .ok_or_else(|| anyhow::anyhow!("{CHECKSUMS_ASSET} has no entry for {asset_name}"))?;
    let actual = sha256_of_file(path)?;
    if actual != expected {
        anyhow::bail!("SHA-256 mismatch for {asset_name}: expected {expected}, got {actual}");
    }
    Ok(())
}

fn verify_minisign(public_key: &str, message: &[u8], signature: &str) -> anyhow::Result<()> {
    let public_key = minisign_verify::PublicKey::from_base64(public_key)
        .context("embedded update public key is malformed")?;
    let signature =
        minisign_verify::Signature::decode(signature).context("release signature is malformed")?;
    public_key
        .verify(message, &signature, false)
        .context("release signature does not verify against the embedded key")
}

impl UpdateManager {
    pub fn new(cx: &mut App) -> Self {
        // Reading the just-updated marker is best-effort: if the local data
        // directory can't be resolved we simply skip the post-update notice
        // rather than panicking at startup.
        let marker_path = Self::updates_dir().ok().map(|d| d.join(".updated_from"));
        let just_updated_from = if let Some(marker_path) = marker_path {
            if marker_path.exists() {
                let version = std::fs::read_to_string(&marker_path).ok();
                if let Err(e) = std::fs::remove_file(&marker_path) {
                    tracing::warn!("Failed to remove update marker file: {}", e);
                }
                cx.new(|_cx| version)
            } else {
                cx.new(|_cx| None)
            }
        } else {
            cx.new(|_cx| None)
        };

        Self {
            pending_update: cx.new(|_cx| None),
            just_updated_from,
        }
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn start_polling(cx: &mut App) {
        cx.spawn(async move |cx| {
            loop {
                cx.update(|cx| {
                    if AppSettings::global(cx).settings.general.check_for_updates {
                        Self::poll_for_updates(cx);
                    }
                });
                cx.background_executor().timer(CHECK_INTERVAL).await;
            }
        })
        .detach();
    }

    fn poll_for_updates(cx: &mut App) {
        let pending_update = Self::global(cx).pending_update.clone();

        cx.spawn(async move |cx| {
            // Check for staged update first
            let staged_exists = Self::staged_binary_path()
                .map(|p| p.exists())
                .unwrap_or(false);

            let latest = match Self::fetch_latest_release().await {
                Ok(latest) => latest,
                Err(e) => {
                    tracing::warn!("update check failed: {e:#}");
                    return Ok::<_, anyhow::Error>(());
                }
            };

            if staged_exists {
                // Already have staged update, just update UI
                if let Some(info) = latest {
                    cx.update(|cx| {
                        pending_update.update(cx, |state, cx| {
                            *state = Some(info);
                            cx.notify();
                        });
                    });
                }
                return Ok(());
            }

            if let Some(release) = latest {
                tracing::info!("Update found: {}, downloading...", release.version);

                // Download in blocking context
                let result = smol::unblock(Self::download_update_to_staging).await;

                match result {
                    Ok(()) => {
                        tracing::info!("Update staged successfully");
                        cx.update(|cx| {
                            pending_update.update(cx, |state, cx| {
                                *state = Some(release);
                                cx.notify();
                            });
                        });
                    }
                    Err(e) => tracing::error!("Failed to stage update: {e:#}"),
                }
            }
            Ok(())
        })
        .detach();
    }

    async fn fetch_latest_release() -> anyhow::Result<Option<ReleaseInfo>> {
        smol::unblock(move || {
            let mut builder = self_update::backends::github::Update::configure();
            builder
                .repo_owner(GITHUB_OWNER)
                .repo_name(GITHUB_REPO)
                .bin_name("shoutingrobin")
                .current_version(env!("CARGO_PKG_VERSION"))
                .show_output(false);

            // A release with no asset for this build is not an update the
            // user can take, so it is not announced as one.
            let asset_name = Self::release_asset_name()?;
            let updater = builder.build()?;
            let release = updater.get_latest_release()?;
            if !release.assets.iter().any(|asset| asset.name == asset_name) {
                return Ok(None);
            }

            let remote_version_str = release
                .version
                .strip_prefix('v')
                .unwrap_or(&release.version);
            let remote_version = Version::parse(remote_version_str)?;
            let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

            if remote_version > current_version {
                Ok(Some(ReleaseInfo {
                    version: release.version.clone(),
                    html_url: Self::changelog_url(&release.version),
                }))
            } else {
                Ok(None)
            }
        })
        .await
    }

    fn updates_dir() -> anyhow::Result<PathBuf> {
        Ok(dirs::data_local_dir()
            .context("Failed to get local data directory")?
            .join("shoutingrobin")
            .join("updates"))
    }

    fn staged_binary_path() -> anyhow::Result<PathBuf> {
        let name = if cfg!(target_os = "windows") {
            "shoutingrobin.exe"
        } else {
            "shoutingrobin"
        };
        Ok(Self::updates_dir()?.join(name))
    }

    /// The exact release asset name for this build, as the release workflow
    /// writes it. Exact, not a substring: on Linux every asset name "ends
    /// with" the empty string, so a substring match could pick the `.deb` or
    /// the checksum file.
    fn release_asset_name() -> anyhow::Result<&'static str> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Ok("shoutingrobin-linux-x86_64"),
            ("windows", "x86_64") => Ok("shoutingrobin-windows-x86_64.exe"),
            ("macos", "aarch64") => Ok("shoutingrobin-macos-arm64.tar.gz"),
            (os, arch) => anyhow::bail!("no release build is published for {os}/{arch}"),
        }
    }

    fn download_update_to_staging() -> anyhow::Result<()> {
        std::fs::create_dir_all(Self::updates_dir()?)?;

        let mut builder = self_update::backends::github::Update::configure();
        builder
            .repo_owner(GITHUB_OWNER)
            .repo_name(GITHUB_REPO)
            .bin_name("shoutingrobin")
            .current_version(env!("CARGO_PKG_VERSION"))
            .no_confirm(true)
            .show_output(false)
            .show_download_progress(false);
        let updater = builder.build()?;
        let release = updater.get_latest_release()?;

        let asset_name = Self::release_asset_name()?;
        let find_asset = |name: &str| {
            release
                .assets
                .iter()
                .find(|a| a.name == name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("release {} has no asset {name}", release.version))
        };
        let asset = find_asset(asset_name)?;
        let checksums = find_asset(CHECKSUMS_ASSET)?;

        let updates = Self::updates_dir()?;
        let tmp_archive_path = updates.join(&asset.name);
        Self::download_asset(&asset.download_url, &tmp_archive_path)?;
        let checksums_path = updates.join(CHECKSUMS_ASSET);
        Self::download_asset(&checksums.download_url, &checksums_path)?;

        // Verify before anything is extracted or executed. The checksum file
        // proves the bytes are the ones the release workflow produced; the
        // signature over that file proves the workflow was ours.
        let verification = (|| -> anyhow::Result<()> {
            let checksums_text = std::fs::read_to_string(&checksums_path)?;
            if let Some(public_key) = UPDATE_PUBLIC_KEY {
                let signature_asset = find_asset(CHECKSUMS_SIGNATURE_ASSET)?;
                let signature_path = updates.join(CHECKSUMS_SIGNATURE_ASSET);
                Self::download_asset(&signature_asset.download_url, &signature_path)?;
                let signature = std::fs::read_to_string(&signature_path)?;
                verify_minisign(public_key, checksums_text.as_bytes(), &signature)?;
                if let Err(e) = std::fs::remove_file(&signature_path) {
                    tracing::warn!("Failed to remove downloaded signature: {}", e);
                }
            } else {
                tracing::warn!(
                    "update signing key not configured; relying on checksums and TLS only"
                );
            }
            verify_sha256(&tmp_archive_path, &asset.name, &checksums_text)
        })();
        if let Err(e) = std::fs::remove_file(&checksums_path) {
            tracing::warn!("Failed to remove downloaded checksums: {}", e);
        }
        if let Err(e) = verification {
            if let Err(remove_error) = std::fs::remove_file(&tmp_archive_path) {
                tracing::warn!("Failed to remove rejected download: {}", remove_error);
            }
            return Err(e.context("downloaded update failed verification"));
        }

        let staged = Self::staged_binary_path()?;
        if cfg!(target_os = "macos") {
            self_update::Extract::from_source(&tmp_archive_path)
                .extract_file(&updates, MACOS_BIN_IN_ARCHIVE)?;
            // For tar archives self_update preserves the full archive path, so
            // the binary lands nested inside the extracted .app bundle. Move it
            // to the flat staged location we relaunch from, then discard the
            // leftover bundle dir.
            std::fs::rename(updates.join(MACOS_BIN_IN_ARCHIVE), &staged)?;
            let bundle = updates.join("Shouting Robin.app");
            if bundle.exists()
                && let Err(e) = std::fs::remove_dir_all(&bundle)
            {
                tracing::warn!("Failed to remove extracted bundle dir: {}", e);
            }
            if let Err(e) = std::fs::remove_file(&tmp_archive_path) {
                tracing::warn!("Failed to remove temporary archive: {}", e);
            }
        } else {
            // The Linux and Windows assets are the bare executable.
            std::fs::rename(&tmp_archive_path, &staged)?;
        }

        // Pin what was verified, so that a file swapped into the (user
        // writable) staging directory between now and the user's click is
        // caught at apply time.
        let digest = sha256_of_file(&staged)?;
        std::fs::write(Self::staged_digest_path()?, digest)?;

        Ok(())
    }

    fn download_asset(url: &str, destination: &std::path::Path) -> anyhow::Result<()> {
        let mut file = std::fs::File::create(destination)?;
        let mut download = self_update::Download::from_url(url);
        download.set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?);
        download.show_progress(false);
        download.download_to(&mut file)?;
        Ok(())
    }

    fn staged_digest_path() -> anyhow::Result<PathBuf> {
        Ok(Self::updates_dir()?.join("shoutingrobin.sha256"))
    }

    pub fn apply_pending_update() -> anyhow::Result<()> {
        let staged = Self::staged_binary_path()?;
        if !staged.exists() {
            anyhow::bail!("No staged update found at {staged:?}");
        }
        let expected = std::fs::read_to_string(Self::staged_digest_path()?)
            .context("staged update has no recorded digest; download it again")?;
        let actual = sha256_of_file(&staged)?;
        if actual != expected.trim() {
            if let Err(e) = std::fs::remove_file(&staged) {
                tracing::warn!("Failed to remove tampered staged update: {}", e);
            }
            anyhow::bail!("staged update does not match the verified download; discarded it");
        }

        if let Err(e) = self_update::self_replace::self_replace(&staged) {
            anyhow::bail!("Failed to apply update: {e}");
        }
        // Written only once the binary is in place, so a failed replace does
        // not announce an update that never happened.
        let current = env!("CARGO_PKG_VERSION").to_string();
        if let Err(e) = std::fs::write(Self::updates_dir()?.join(".updated_from"), &current) {
            tracing::warn!("Failed to write update marker file: {}", e);
        }

        for leftover in [staged, Self::staged_digest_path()?] {
            if let Err(e) = std::fs::remove_file(&leftover) {
                tracing::warn!("Failed to remove staged update file {leftover:?}: {}", e);
            }
        }
        tracing::info!("Update applied, restarting...");
        Self::restart_app()
    }

    fn restart_app() -> anyhow::Result<()> {
        let exe = std::env::current_exe().context("Failed to get current exe path")?;

        #[cfg(target_os = "macos")]
        {
            // On macOS, restart the .app bundle if we're inside one
            if let Some(app_bundle) = exe
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                && app_bundle.extension().is_some_and(|ext| ext == "app")
            {
                if let Err(e) = std::process::Command::new("open").arg(app_bundle).spawn() {
                    tracing::error!("Failed to restart app bundle: {}", e);
                }
                std::process::exit(0);
            }
        }

        if let Err(e) = std::process::Command::new(exe).spawn() {
            tracing::error!("Failed to restart application: {}", e);
        }
        std::process::exit(0);
    }

    pub fn changelog_url(version: &str) -> String {
        format!(
            "https://github.com/{}/{}/releases/tag/{}",
            GITHUB_OWNER, GITHUB_REPO, version
        )
    }
}
