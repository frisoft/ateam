# Use: nix develop
#
# To update the lock file, run:
# nix flake update

{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/master"; # very latest packages
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShell = with pkgs;
          mkShell {
            buildInputs = with pkgs;
              [
                rustup
                cargo
                rustc
                libiconv
                pkg-config
                openssl
                # tools for ./maintenance.sh
                cargo-udeps
                cargo-outdated
                cargo-audit
              ] ++ lib.optionals pkgs.hostPlatform.isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
                darwin.apple_sdk.frameworks.CoreServices
                darwin.apple_sdk.frameworks.Security
              ];
          };
      });
}
