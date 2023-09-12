{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.05";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk?rev=3be4895447e6f0f12cba5d893329ac4c52e553e2";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      inherit (pkgs) lib callPackage rust-bin mkShell;
      inherit (builtins) listToAttrs fromTOML readFile;
      inherit (lib.sources) sourceByRegex;

      toolchain = rust-bin.stable.latest.default;
      msrv = (fromTOML (readFile ./Cargo.toml)).package.rust-version;
      msrvToolchain = rust-bin.stable."${msrv}".default;
      miriToolchain = rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
        extensions = [ "miri" "rust-src" ];
      });

      naersk' = callPackage naersk {
        rustc = toolchain;
        cargo = toolchain;
      };
      msrvNaersk = callPackage naersk {
        rustc = msrvToolchain;
        cargo = msrvToolchain;
      };
      src = sourceByRegex ./. ["Cargo.*" "(src|tests|bitbuffer_derive|benches)(/.*)?"];

      naerskOpt = {
        pname = "bitbuffer";
        root = src;
        copySources = ["bitbuffer_derive"];
      };
    in rec {
      packages = {
        check = naersk'.buildPackage (naerskOpt // {
          mode = "check";
        });
        test = naersk'.buildPackage (naerskOpt // {
          mode = "test";
          cargoTestOptions = x: x ++ ["--all-features"];
        });
        clippy = naersk'.buildPackage (naerskOpt // {
          mode = "clippy";
        });
        fmt = naersk'.buildPackage (naerskOpt // {
          mode = "fmt";
        });
        msrv = msrvNaersk.buildPackage (naerskOpt // {
          mode = "check";
        });
      };

      miriTargets = [
        "x86_64-unknown-linux-musl" # little-endian
        "mips64-unknown-linux-gnuabi64" # big-endian
      ];

      devShells = let
        tools = with pkgs; [
          bacon
          cargo-edit
          cargo-outdated
          (writeShellApplication {
            name = "cargo-expand";
            runtimeInputs = [cargo-expand toolchain];
            text = ''
              # shellcheck disable=SC2068
              RUSTC_BOOTSTRAP=1 cargo-expand $@
            '';
          })
        ];
      in {
        default = mkShell {
          nativeBuildInputs = [toolchain] ++ tools;
        };
        msrv = mkShell {
          nativeBuildInputs = [msrvToolchain] ++ tools;
        };
        miri = mkShell {
          nativeBuildInputs = [miriToolchain] ++ tools;
        };
      };
    });
}
