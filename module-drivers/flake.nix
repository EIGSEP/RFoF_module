{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    fenix,
    flake-utils,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [fenix.overlays.default];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Setup the rust toolchain
        rustToolchain = fenix.packages.${system}.complete.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
        ];
        craneLib =
          (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Project setup
        crate = craneLib.crateNameFromCargoToml {cargoToml = ./Cargo.toml;};
        projectName = crate.pname;
        projectVersion = crate.version;

        # Python setup
        pythonVersion = pkgs.python3;
        # Unfortunatley backwards from ${system}, which is x86_64_linux
        wheelTail = "cp38-abi3-linux_x86_64";
        wheelName = "${projectName}-${projectVersion}-${wheelTail}.whl";

        crateCfg = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          nativeBuildInputs = [pythonVersion];
        };

        # Build the library, then re-use the target dir to generate the wheel file with maturin
        crateWheel =
          (craneLib.buildPackage (crateCfg
            // {
              pname = projectName;
              version = projectVersion;
            }))
          .overrideAttrs (old: {
            nativeBuildInputs = old.nativeBuildInputs ++ [pkgs.maturin];
            buildPhase =
              old.buildPhase
              + ''
                maturin build --offline --target-dir ./target
              '';
            installPhase =
              old.installPhase
              + ''
                ls target/wheels/
                cp target/wheels/${wheelName} $out/
              '';
          });
      in rec {
        packages = {
          default = crateWheel; # The wheel itself

          # A python version with the library installed and some utilities
          pythonEnv =
            pythonVersion.withPackages
            (ps: [(lib.pythonPackage ps)] ++ (with ps; [jupyterlab]));
        };

        lib = {
          # To use in other builds with the "withPackages" call
          pythonPackage = ps:
            ps.buildPythonPackage {
              pname = projectName;
              format = "wheel";
              version = projectVersion;
              src = "${crateWheel}/${wheelName}";
              doCheck = false;
              pythonImportsCheck = [projectName];
            };
        };

        devShells = rec {
          rust = pkgs.mkShell {
            name = "rust-env";
            src = ./.;
            nativeBuildInputs = with pkgs; [pkg-config rustToolchain];
            buildInputs = with pkgs; [rust-analyzer-nightly];
          };
          python = pkgs.mkShell {
            name = "python-env";
            src = ./.;
            nativeBuildInputs = [packages.pythonEnv];
          };
          default = rust;
        };

        # nix run to get a jupyterlab instance
        apps = rec {
          jupyterlab = {
            type = "app";
            program = "${packages.pythonEnv}/bin/jupyter-lab";
          };
          default = jupyterlab;
        };
      }
    );
}
