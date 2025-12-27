{
  description = "Unified Git/JJ Starship prompt module optimized for latency";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    {
      overlays.default = final: prev: {
        jj-starship = self.packages.${final.system}.jj-starship;
        jj-starship-no-git = self.packages.${final.system}.jj-starship-no-git;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

        # Version suffix: use git rev if available, otherwise "dirty"
        versionSuffix = if self ? rev then "-${builtins.substring 0 7 self.rev}" else "-dirty";

        # Common source filtering for all variants
        src = pkgs.lib.fileset.toSource {
          root = ./.;
          fileset = pkgs.lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./src
            ./build.rs
          ];
        };

        # Shared meta for all variants
        meta = with pkgs.lib; {
          description = "Unified Git/JJ Starship prompt module optimized for latency";
          homepage = "https://github.com/dmmulroy/jj-starship";
          changelog = "https://github.com/dmmulroy/jj-starship/releases/tag/v${version}";
          license = licenses.mit;
          maintainers = [ ]; # TODO: Add maintainer once registered in nixpkgs
          mainProgram = "jj-starship";
          platforms = platforms.unix;
        };

        # Build package with configurable features
        mkJjStarship =
          {
            withGit ? true,
          }:
          pkgs.rustPlatform.buildRustPackage {
            pname = "jj-starship" + pkgs.lib.optionalString (!withGit) "-no-git";
            version = "${version}${versionSuffix}";

            inherit src meta;

            cargoLock.lockFile = ./Cargo.lock;

            buildNoDefaultFeatures = !withGit;

            nativeBuildInputs = [ pkgs.pkg-config ];

            buildInputs =
              with pkgs;
              [
                openssl
                zlib
              ]
              ++ lib.optionals withGit [ libgit2 ]
              ++ lib.optionals stdenv.hostPlatform.isDarwin [
                # Security.framework - TLS/SSL and cryptographic operations for HTTPS git
                # SystemConfiguration.framework - Network configuration (proxy, DNS)
                apple-sdk
                # libiconv - Character encoding conversion (separate from glibc on macOS)
                libiconv
              ];

            doCheck = true;
          };
      in
      {
        packages = {
          jj-starship = mkJjStarship { withGit = true; };
          jj-starship-no-git = mkJjStarship { withGit = false; };
          default = self.packages.${system}.jj-starship;
        };

        # Checks run by `nix flake check` - used in CI to verify builds
        checks = {
          # Ensure both package variants build successfully
          jj-starship = self.packages.${system}.jj-starship;
          jj-starship-no-git = self.packages.${system}.jj-starship-no-git;
        };

        # Development shell with Rust tooling
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.jj-starship ];
          packages = with pkgs; [
            rust-analyzer
            rustfmt
            clippy
            cargo-watch
          ];
        };

        # Formatter for `nix fmt`
        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
