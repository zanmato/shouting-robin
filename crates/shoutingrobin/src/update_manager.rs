use crate::app_settings::AppSettings;
use anyhow::Context as _;
use gpui_kit::{App, AppContext, Entity, Global, Task};
use semver::Version;
use std::path::PathBuf;
use std::time::Duration;

const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);
const UPDATED_FROM_MARKER: &str = ".updated_from";
const GITHUB_OWNER: &str = "zanmato";
const GITHUB_REPO: &str = "shouting-robin";

// The macOS .app bundle ships the executable under this path (see the release
// workflow). The name contains a space because the bundle is user-facing.
const MACOS_BIN_IN_ARCHIVE: &str = "Shouting Robin.app/Contents/MacOS/Shouting Robin";

/// Release assets the workflow publishes next to the binaries: the SHA-256 of
/// every asset, and a minisign signature over that file.
const CHECKSUMS_ASSET: &str = "SHA256SUMS";
const CHECKSUMS_SIGNATURE_ASSET: &str = "SHA256SUMS.minisig";

/// The minisign public key releases are signed with. The secret half is the
/// `MINISIGN_SECRET_KEY` repository secret, which the release workflow signs
/// `SHA256SUMS` with (see `scripts/generate-update-key.sh`).
const UPDATE_PUBLIC_KEY: Option<&str> =
    Some("RWTw5QZHMdaXJeK+RkjKnGxEpQKHVWXqsA3o3Poy75et7vFbMJhMtAW9");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateState {
    UpToDate,
    /// A newer release exists and has not been downloaded.
    Available(String),
    /// A newer release exists, but this install cannot replace its own
    /// executable (a system package owns it), so the user is sent to the
    /// release page instead.
    Manual(String),
    Downloading(String),
    /// The release is downloaded, verified and applied by restarting.
    Ready(String),
}

pub struct UpdateManager {
    pub state: Entity<UpdateState>,
    /// The version this install ran before the update applied on the previous
    /// run, present only on the first launch after updating.
    pub updated_from: Option<String>,
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
        let updated_from = match Self::updates_dir() {
            Ok(updates_dir) => {
                let marker_path = updates_dir.join(UPDATED_FROM_MARKER);
                match std::fs::read_to_string(&marker_path) {
                    Ok(version) => {
                        if let Err(e) = std::fs::remove_file(&marker_path) {
                            tracing::warn!("Failed to remove update marker file: {}", e);
                        }
                        Some(version)
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                    Err(e) => {
                        tracing::warn!("Failed to read update marker file: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to resolve updates directory: {e:#}");
                None
            }
        };

        Self {
            state: cx.new(|_cx| UpdateState::UpToDate),
            updated_from,
        }
    }

    /// `None` when no manager is installed, as in tests that build the app
    /// without running `main`.
    pub fn try_global(cx: &App) -> Option<&Self> {
        cx.try_global::<Self>()
    }

    pub fn start_polling(cx: &mut App) {
        if let Err(e) = Self::release_asset_name() {
            tracing::info!("update checks are disabled: {e:#}");
            return;
        }

        cx.spawn(async move |cx| {
            loop {
                let check = cx.update(|cx| {
                    AppSettings::global(cx)
                        .settings
                        .general
                        .check_for_updates
                        .then(|| Self::try_global(cx).map(|this| this.state.clone()))
                        .flatten()
                });
                if let Some(state) = check {
                    match smol::unblock(Self::check_latest_release).await {
                        Ok(Some(found)) => {
                            cx.update(|cx| {
                                state.update(cx, |state, cx| {
                                    // A running download reports its own result.
                                    if !matches!(state, UpdateState::Downloading(_)) {
                                        *state = found;
                                        cx.notify();
                                    }
                                });
                            });
                        }
                        Ok(None) => {}
                        Err(e) => tracing::warn!("update check failed: {e:#}"),
                    }
                }
                cx.background_executor().timer(CHECK_INTERVAL).await;
            }
        })
        .detach();
    }

    /// The latest release when it is newer than this build and carries an
    /// asset for it. A release with no asset for this build is not an update
    /// the user can take, so it is not announced as one.
    fn newer_release() -> anyhow::Result<Option<self_update::update::Release>> {
        let asset_name = Self::release_asset_name()?;
        let release = self_update::backends::github::Update::configure()
            .repo_owner(GITHUB_OWNER)
            .repo_name(GITHUB_REPO)
            .bin_name("shoutingrobin")
            .current_version(env!("CARGO_PKG_VERSION"))
            .show_output(false)
            .build()?
            .get_latest_release()?;
        if !release.assets.iter().any(|asset| asset.name == asset_name) {
            return Ok(None);
        }

        let remote_version = Version::parse(
            release
                .version
                .strip_prefix('v')
                .unwrap_or(&release.version),
        )?;
        Ok((remote_version > Version::parse(env!("CARGO_PKG_VERSION"))?).then_some(release))
    }

    fn check_latest_release() -> anyhow::Result<Option<UpdateState>> {
        let Some(release) = Self::newer_release()? else {
            return Ok(None);
        };
        Ok(Some(if !Self::executable_is_replaceable() {
            UpdateState::Manual(release.version)
        } else if Self::staged_binary_path(&release.version)?.exists() {
            UpdateState::Ready(release.version)
        } else {
            UpdateState::Available(release.version)
        }))
    }

    /// Whether the running executable can be swapped in place, which takes
    /// creating a file next to it. False for a `.deb` install in `/usr/bin`.
    fn executable_is_replaceable() -> bool {
        let Some(directory) = std::env::current_exe()
            .ok()
            .and_then(|executable| Some(executable.parent()?.to_path_buf()))
        else {
            return false;
        };
        let probe = directory.join(format!(
            ".shoutingrobin-update-probe-{}",
            std::process::id()
        ));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe)
        {
            Ok(_) => {
                if let Err(e) = std::fs::remove_file(&probe) {
                    tracing::warn!("Failed to remove update probe file: {}", e);
                }
                true
            }
            Err(_) => false,
        }
    }

    /// Download the available release. The state moves to `Downloading` right
    /// away and to `Ready` once the executable is verified and staged; on
    /// failure it falls back to `Available` and the error is returned for the
    /// caller to show.
    pub fn download_update(cx: &mut App) -> Task<anyhow::Result<()>> {
        let Some(state) = Self::try_global(cx).map(|this| this.state.clone()) else {
            return Task::ready(Err(anyhow::anyhow!("Updates are not available")));
        };
        let UpdateState::Available(version) = state.read(cx).clone() else {
            return Task::ready(Ok(()));
        };
        state.update(cx, |state, cx| {
            *state = UpdateState::Downloading(version.clone());
            cx.notify();
        });

        cx.spawn(async move |cx| {
            let result = smol::unblock(Self::stage_latest_release).await;
            cx.update(|cx| {
                state.update(cx, |state, cx| {
                    *state = match &result {
                        Ok(Some(staged_version)) => UpdateState::Ready(staged_version.clone()),
                        Ok(None) => UpdateState::UpToDate,
                        Err(_) => UpdateState::Available(version),
                    };
                    cx.notify();
                });
            });
            result.map(|_| ())
        })
    }

    fn updates_dir() -> anyhow::Result<PathBuf> {
        Ok(dirs::data_local_dir()
            .context("Failed to get local data directory")?
            .join("shoutingrobin")
            .join("updates"))
    }

    /// Staged executables are kept per version, so one downloaded for a
    /// release that has since been superseded is never applied.
    fn staged_binary_path(version: &str) -> anyhow::Result<PathBuf> {
        let name = if cfg!(target_os = "windows") {
            "shoutingrobin.exe"
        } else {
            "shoutingrobin"
        };
        Ok(Self::updates_dir()?.join(version).join(name))
    }

    fn staged_digest_path(version: &str) -> anyhow::Result<PathBuf> {
        Ok(Self::updates_dir()?
            .join(version)
            .join("shoutingrobin.sha256"))
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

    /// Download the latest release, when it is newer than this build, verify
    /// it and stage its executable. Returns the staged version.
    fn stage_latest_release() -> anyhow::Result<Option<String>> {
        let Some(release) = Self::newer_release()? else {
            return Ok(None);
        };
        let staged = Self::staged_binary_path(&release.version)?;
        if staged.exists() {
            return Ok(Some(release.version));
        }

        tracing::info!("Downloading update {}", release.version);
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

        // Starting from an empty directory drops releases staged earlier and
        // never applied. Everything is downloaded and unpacked in a scratch
        // directory and only the verified executable is renamed into place,
        // so an interrupted download never leaves a file at the staged path.
        let updates = Self::updates_dir()?;
        if updates.exists() {
            std::fs::remove_dir_all(&updates)?;
        }
        let download_dir = updates.join("download");
        std::fs::create_dir_all(&download_dir)?;

        let archive_path = download_dir.join(&asset.name);
        Self::download_asset(&asset.download_url, &archive_path)?;
        let checksums_path = download_dir.join(CHECKSUMS_ASSET);
        Self::download_asset(&checksums.download_url, &checksums_path)?;

        // Verify before anything is extracted or executed. The checksum file
        // proves the bytes are the ones the release workflow produced; the
        // signature over that file proves the workflow was ours.
        let verification = (|| -> anyhow::Result<()> {
            let checksums_text = std::fs::read_to_string(&checksums_path)?;
            if let Some(public_key) = UPDATE_PUBLIC_KEY {
                let signature_asset = find_asset(CHECKSUMS_SIGNATURE_ASSET)?;
                let signature_path = download_dir.join(CHECKSUMS_SIGNATURE_ASSET);
                Self::download_asset(&signature_asset.download_url, &signature_path)?;
                let signature = std::fs::read_to_string(&signature_path)?;
                verify_minisign(public_key, checksums_text.as_bytes(), &signature)?;
            } else {
                tracing::warn!(
                    "update signing key not configured; relying on checksums and TLS only"
                );
            }
            verify_sha256(&archive_path, &asset.name, &checksums_text)
        })();
        if let Err(e) = verification {
            if let Err(remove_error) = std::fs::remove_dir_all(&download_dir) {
                tracing::warn!("Failed to remove rejected download: {}", remove_error);
            }
            return Err(e.context("downloaded update failed verification"));
        }

        let verified = if cfg!(target_os = "macos") {
            // For tar archives self_update preserves the full archive path, so
            // the binary lands nested inside the extracted .app bundle.
            self_update::Extract::from_source(&archive_path)
                .extract_file(&download_dir, MACOS_BIN_IN_ARCHIVE)?;
            download_dir.join(MACOS_BIN_IN_ARCHIVE)
        } else {
            // The Linux and Windows assets are the bare executable.
            archive_path
        };

        if let Some(staged_dir) = staged.parent() {
            std::fs::create_dir_all(staged_dir)?;
        }
        std::fs::rename(&verified, &staged)?;
        if let Err(e) = std::fs::remove_dir_all(&download_dir) {
            tracing::warn!("Failed to remove update download directory: {}", e);
        }

        // Pin what was verified, so that a file swapped into the (user
        // writable) staging directory between now and the user's click is
        // caught at apply time.
        let digest = sha256_of_file(&staged)?;
        std::fs::write(Self::staged_digest_path(&release.version)?, digest)?;

        tracing::info!("Update {} staged", release.version);
        Ok(Some(release.version))
    }

    fn download_asset(url: &str, destination: &std::path::Path) -> anyhow::Result<()> {
        let mut file = std::fs::File::create(destination)?;
        let mut download = self_update::Download::from_url(url);
        download.set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?);
        download.show_progress(false);
        download.download_to(&mut file)?;
        Ok(())
    }

    /// Replace the running executable with the staged one and restart. On
    /// success the app is quitting when this returns.
    pub fn apply_pending_update(cx: &mut App) -> anyhow::Result<()> {
        let state = Self::try_global(cx)
            .context("Updates are not available")?
            .state
            .clone();
        let UpdateState::Ready(version) = state.read(cx).clone() else {
            anyhow::bail!("No update has been downloaded");
        };
        let staged = Self::staged_binary_path(&version)?;
        if !staged.exists() {
            anyhow::bail!("No staged update found at {staged:?}");
        }
        let expected = std::fs::read_to_string(Self::staged_digest_path(&version)?)
            .context("staged update has no recorded digest; download it again")?;
        let actual = sha256_of_file(&staged)?;
        if actual != expected.trim() {
            if let Err(e) = std::fs::remove_file(&staged) {
                tracing::warn!("Failed to remove tampered staged update: {}", e);
            }
            // Back to offering the download, which fetches a clean copy.
            state.update(cx, |state, cx| {
                *state = UpdateState::Available(version);
                cx.notify();
            });
            anyhow::bail!("staged update does not match the verified download; discarded it");
        }

        // Resolved before the swap: on Linux the path of a replaced executable
        // reads back with a " (deleted)" suffix.
        let executable = std::env::current_exe().context("Failed to get current exe path")?;

        self_update::self_replace::self_replace(&staged)
            .with_context(|| format!("Failed to replace {}", executable.display()))?;

        // Written only once the binary is in place, so a failed replace does
        // not announce an update that never happened.
        if let Err(e) = std::fs::write(
            Self::updates_dir()?.join(UPDATED_FROM_MARKER),
            env!("CARGO_PKG_VERSION"),
        ) {
            tracing::warn!("Failed to write update marker file: {}", e);
        }
        if let Some(staged_dir) = staged.parent()
            && let Err(e) = std::fs::remove_dir_all(staged_dir)
        {
            tracing::warn!("Failed to remove staged update {staged_dir:?}: {}", e);
        }

        tracing::info!("Update applied, restarting...");
        // macOS relaunches the enclosing .app bundle, which GPUI finds itself.
        if !cfg!(target_os = "macos") {
            cx.set_restart_path(executable);
        }
        cx.restart();
        Ok(())
    }

    pub fn changelog_url(version: &str) -> String {
        format!(
            "https://github.com/{}/{}/releases/tag/{}",
            GITHUB_OWNER, GITHUB_REPO, version
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A throwaway key pair made for this test, not the release key.
    const TEST_PUBLIC_KEY: &str = "RWSuP7TLTlfXLIwvb3Ojg/ZI+g2/fXCh+k87Xwq6gKZ5En2r4pCpPrCZ";
    const TEST_CHECKSUMS: &str = "abc  file\n";
    const TEST_SIGNATURE: &str = "untrusted comment: signature from minisign secret key
RUSuP7TLTlfXLMbHaCt8CBuWVm88JdB+bbtplOR4t7l6bmUjC2sSMb5o0XkolEKyWloUJiVzo61giuxhBDLpEk1kxENxgV8ADQg=
trusted comment: timestamp:1791535679\tfile:SHA256SUMS\thashed
Pdfw32iU8mDa1Y1lt1/n44v5SxpwEAVjKpuw1FmMPw9GT0oti5Qc/7fZgkKkU5VuX78mPJF1tv5x7q53zFMmDw==
";

    #[test]
    fn embedded_public_key_parses() {
        if let Some(public_key) = UPDATE_PUBLIC_KEY {
            minisign_verify::PublicKey::from_base64(public_key)
                .expect("UPDATE_PUBLIC_KEY should be a minisign public key");
        }
    }

    #[test]
    fn minisign_accepts_only_the_signed_checksums() {
        verify_minisign(TEST_PUBLIC_KEY, TEST_CHECKSUMS.as_bytes(), TEST_SIGNATURE)
            .expect("signature made by minisign -S should verify");
        assert!(verify_minisign(TEST_PUBLIC_KEY, b"abd  file\n", TEST_SIGNATURE).is_err());
    }

    #[test]
    fn sha256_must_match_the_entry_for_the_asset() {
        let path =
            std::env::temp_dir().join(format!("shoutingrobin-update-test-{}", std::process::id()));
        std::fs::write(&path, b"hello").expect("write asset");
        let digest = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

        verify_sha256(&path, "asset", &format!("{digest}  asset\n")).expect("matching digest");
        assert!(verify_sha256(&path, "asset", &format!("{digest}  other\n")).is_err());
        assert!(verify_sha256(&path, "asset", &format!("{}  asset\n", "0".repeat(64))).is_err());
        std::fs::remove_file(&path).expect("remove asset");
    }
}
