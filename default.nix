{ lib, rustPlatform, pkg-config, darwin }:

rustPlatform.buildRustPackage rec {
  pname = "ateam";
  version = "1.0.10";

  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };

  nativeBuildInputs = [ pkg-config ];

  meta = with lib; {
    description = "ateam: The tool that helps optimize the code review process";
    license = licenses.mit;
    maintainers = with maintainers; [ "frisoft" ];
  };
}
