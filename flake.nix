# Use: nix develop
#
# To update the lock file, run:
# nix flake update

{
  description = "ateam: The tool that helps optimize the code review process";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        ateam = import ./default.nix { inherit pkgs; };
      in {
        packages.default = ateam;

        devShell = pkgs.mkShell {
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
            ] ++ ateam.buildInputs;
        };
      });
}
