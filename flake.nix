# Use: nix develop
#
# To update the lock file, run:
# nix flake update

{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        deps = with pkgs;
          [ cargo rustc pkg-config openssl cacert libiconv ]
          ++ pkgs.lib.optionals pkgs.hostPlatform.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.darwin.apple_sdk.frameworks.CoreServices
            pkgs.darwin.apple_sdk.frameworks.Security
          ];
      in {
        devShell = with pkgs;
          mkShell {
            buildInputs = with pkgs;
              [
                # rustup
                # cargo
                # rustc
                # libiconv
                # pkg-config
                # openssl
                # rust-analyzer <-- This does not work. You need to run `rustup component add rust-analyzer`
                # tools for ./maintenance.sh
                cargo-udeps
                cargo-outdated
                cargo-audit
                cargo-dist
                cargo-release
                clippy
              ] ++ deps;
          };
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "ateam";
          version = "1.0.7";
          src = ./.;
          buildInputs = deps;

          buildPhase = ''
            export CARGO_NET_GIT_FETCH_WITH_CLI=true
            export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
            export CARGO_HOME=$TMPDIR/cargo
            cargo build --release
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/ateam $out/bin/
          '';

          meta = with pkgs.lib; {
            description =
              "ateam. The tool that helps optimize the code review process.";
            license = licenses.mit;
            maintainers = with maintainers; [ "frisoft" ];
          };
        };
      });
}
