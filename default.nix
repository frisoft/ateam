{ lib, rustPlatform, pkg-config, darwin }:

let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
rustPlatform.buildRustPackage rec {
  pname = "ateam";
  version = cargoToml.package.version;

  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };

  nativeBuildInputs = [ pkg-config ];

  meta = with lib; {
    description = "ateam: The tool that helps optimize the code review process";
    license = licenses.mit;
    maintainers = with maintainers; [ "frisoft" ];
  };
}
