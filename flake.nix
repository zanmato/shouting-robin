{
  description = "Shouting Robin, an ecommerce-first SEO crawler";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forEachSystem = function: nixpkgs.lib.genAttrs systems (system: function nixpkgs.legacyPackages.${system});

      # Built from the package set it is given, so the overlay links against
      # the system's own glibc, which its graphics drivers were built with.
      shoutingrobinFor = pkgs: pkgs.callPackage ./nix/package.nix { craneLib = crane.mkLib pkgs; };
    in
    {
      packages = forEachSystem (pkgs: rec {
        shoutingrobin = shoutingrobinFor pkgs;
        default = shoutingrobin;
      });

      overlays.default = final: prev: { shoutingrobin = shoutingrobinFor final; };

      devShells = forEachSystem (
        pkgs:
        let
          shoutingrobin = self.packages.${pkgs.stdenv.hostPlatform.system}.shoutingrobin;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ shoutingrobin ];
            packages = with pkgs; [
              cargo
              clippy
              rustc
              rustfmt
              rust-analyzer
            ];
            # cargo run starts Shouting Robin from target/, which has no rpath to the
            # libraries GPUI loads at runtime.
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath shoutingrobin.runtimeLibraries;
          };
        }
      );

      formatter = forEachSystem (pkgs: pkgs.nixfmt);
    };
}
