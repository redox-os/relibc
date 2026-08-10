{
  inputs = {
    self.submodules = true;

    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    redoxer = {
      url = "gitlab:redox-os/redoxer?host=gitlab.redox-os.org";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
        treefmt-nix.follows = "treefmt-nix";
      };
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      redoxer,
      rust-overlay,
      treefmt-nix,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      overlays = [ rust-overlay.overlays.default ];

      pkgsFor =
        system:
        import nixpkgs {
          inherit system overlays;
        };

      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f (pkgsFor system));
      manifestPkgs = pkgs: (pkgs.lib.importTOML ./Cargo.toml).package;
      rustToolchain = pkgs: pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      treefmtEval = forAllSystems (
        pkgs:
        treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs = {
            nixfmt.enable = true;
            rustfmt = {
              enable = true;
              edition = (manifestPkgs pkgs).edition;
              package = rustToolchain pkgs;
            };
          };
          settings.formatter.rustfmt.excludes = [
            "dlmalloc-rs/*"
          ];
        }
      );

      gccWrapped =
        pkgs:
        pkgs.writeShellScriptBin "${pkgs.stdenv.hostPlatform.system}-gnu-gcc" ''
          ${pkgs.gcc-unwrapped}/bin/gcc $@
        '';
      arWrapped =
        pkgs:
        pkgs.writeShellScriptBin "${pkgs.stdenv.hostPlatform.system}-gnu-ar" ''
          ${pkgs.gcc-unwrapped}/bin/gcc-ar $@
        '';
      nmWrapped =
        pkgs:
        pkgs.writeShellScriptBin "${pkgs.stdenv.hostPlatform.system}-gnu-nm" ''
          ${pkgs.gcc-unwrapped}/bin/gcc-nm $@
        '';
      ldWrapped =
        pkgs:
        pkgs.writeShellScriptBin "${pkgs.stdenv.hostPlatform.system}-gnu-ld" ''
          ${pkgs.binutils-unwrapped}/bin/ld $@
        '';
    in
    {
      packages = forAllSystems (
        pkgs:
        let
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain pkgs;
            rustc = rustToolchain pkgs;
          };
          manifest = manifestPkgs pkgs;
        in
        {
          default = rustPlatform.buildRustPackage (finalAttrs: {
            pname = manifest.name;
            version = manifest.version;
            src = ./.;

            cargoDeps = pkgs.symlinkJoin {
              name = "cargo-vendor-dir";

              paths = [
                (rustPlatform.importCargoLock {
                  lockFile = ./Cargo.lock;
                  outputHashes = {
                    "object-0.36.7" = "sha256-PZ+DuPRiXpHg9hz1Qnh9hrdcFIrhtWiNeIFDLLfq8Jo=";
                    "redox-buffer-pool-0.6.0" = "sha256-nFNJ3oG4k2nzheyjZvKUKXVU5nhEikN79P9HnAAGalM=";
                    "redox-rings-0.1.0" = "sha256-sgWOm6MIf79FQ1i+7+G1fig7nvecD4//NJBIh6V3p8A=";
                    "redox_syscall-0.9.4" = "sha256-xOFLXdYWS9+fxutKtDRRGqtV0MlBdzW9OcyNhoCzX1M=";
                  };
                })
                pkgs.rustPlatform.rustVendorSrc
              ];
            };

            postPatch = ''
              patchShebangs --build renamesyms.sh stripcore.sh check.sh fmt.sh
            '';

            nativeBuildInputs = with pkgs; [
              rust-cbindgen
              (gccWrapped pkgs)
              (arWrapped pkgs)
              (nmWrapped pkgs)
              (ldWrapped pkgs)
            ];

            doCheck = false;

            installPhase = ''
              runHook preInstall

              mkdir -p $out
              DESTDIR=$out make $installFlags install

              runHook postInstall
            '';

            hardeningDisable = [ "all" ];
          });
        }
      );

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [
            self.packages.${pkgs.stdenv.hostPlatform.system}.default
          ];

          packages = [
            pkgs.expect
            pkgs.nixd
            redoxer.packages.${pkgs.stdenv.hostPlatform.system}.default
          ];

          hardeningDisable = [ "all" ];
        };
      });

      formatter = forAllSystems (
        pkgs: treefmtEval.${pkgs.stdenv.hostPlatform.system}.config.build.wrapper
      );

      checks = forAllSystems (pkgs: {
        formatting = treefmtEval.${pkgs.stdenv.hostPlatform.system}.config.build.check self;
      });
    };
}
