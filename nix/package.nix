{
  lib,
  craneLib,
  pkg-config,
  alsa-lib,
  fontconfig,
  freetype,
  libxkbcommon,
  libx11,
  libxcb,
  libxcursor,
  libxi,
  libxrandr,
  vulkan-loader,
  wayland,
  zlib,
}:

let
  root = ../.;

  # Sources the build reads. Docs, the website and the Nix files stay out, so
  # editing them does not rebuild Shouting Robin.
  src = lib.fileset.toSource {
    inherit root;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../crates
    ];
  };

  cargoToml = lib.importTOML ../Cargo.toml;

  # GPUI loads these with dlopen, so the linker never records them. They go in
  # an rpath rather than a wrapper's LD_LIBRARY_PATH, which every program
  # Shouting Robin starts would inherit.
  runtimeLibraries = [
    vulkan-loader
    wayland
    libxkbcommon
    libx11
    libxcb
    libxcursor
    libxi
    libxrandr
  ];

  commonArgs = {
    inherit src;
    pname = "shoutingrobin";
    version = cargoToml.workspace.package.version;
    strictDeps = true;
    cargoExtraArgs = "--package shoutingrobin";

    nativeBuildInputs = [ pkg-config ];

    buildInputs = [
      alsa-lib
      fontconfig
      freetype
      zlib
    ]
    ++ runtimeLibraries;
  };

  # Every dependency built once and reused until Cargo.lock changes.
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    # The tests crawl local servers and start Chrome, so they run in CI instead.
    doCheck = false;

    NIX_LDFLAGS = "-rpath ${lib.makeLibraryPath runtimeLibraries}";
    # The fixup's patchelf --shrink-rpath keeps only directories of libraries
    # the binary links, which would drop the ones above.
    dontPatchELF = true;

    passthru = { inherit runtimeLibraries; };

    postInstall = ''
      install -Dm644 crates/shoutingrobin/resources/shoutingrobin.desktop \
        $out/share/applications/shoutingrobin.desktop
      install -Dm644 crates/shoutingrobin/resources/shoutingrobin-icon-512x512.png \
        $out/share/icons/hicolor/512x512/apps/shoutingrobin.png
    '';

    meta = {
      description = cargoToml.workspace.package.description;
      homepage = "https://github.com/zanmato/shouting-robin";
      license = lib.licenses.asl20;
      mainProgram = "shoutingrobin";
      platforms = lib.platforms.linux;
    };
  }
)
