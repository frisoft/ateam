{ lib, rustPlatform, pkg-config, darwin }:

rustPlatform.buildRustPackage rec {
  pname = "ateam";
  version = "1.0.9";

  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ ] ++ lib.optionals darwin.apple_sdk.stdenv.isDarwin [
    darwin.apple_sdk.frameworks.SystemConfiguration
    darwin.apple_sdk.frameworks.CoreServices
    darwin.apple_sdk.frameworks.Security
  ];

  meta = with lib; {
    description = "ateam: The tool that helps optimize the code review process";
    license = licenses.mit;
    maintainers = with maintainers; [ "frisoft" ];
  };
}
