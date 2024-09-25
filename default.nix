{ pkgs ? import <nixpkgs> { } }:

let
  deps = with pkgs;
    [ cargo rustc pkg-config openssl cacert libiconv ]
    ++ pkgs.lib.optionals pkgs.hostPlatform.isDarwin [
      pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
      pkgs.darwin.apple_sdk.frameworks.CoreServices
      pkgs.darwin.apple_sdk.frameworks.Security
    ];
in pkgs.stdenv.mkDerivation {
  pname = "ateam";
  version = "1.0.9";
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
    description = "ateam: The tool that helps optimize the code review process";
    license = licenses.mit;
    maintainers = with maintainers; [ "frisoft" ];
  };
}
