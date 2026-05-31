{ lib, rustPlatform, makeWrapper, pkg-config, openssl }:

rustPlatform.buildRustPackage {
  pname = "lmha3";
  version = "0.0.14";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ makeWrapper pkg-config ];
  buildInputs = [ openssl ];

  postInstall = ''
    mkdir -p $out/share/lmha3
    cp -r server/public $out/share/lmha3/public
    wrapProgram $out/bin/server \
      --set-default LMHA3_PUBLIC_DIR "$out/share/lmha3/public"
  '';

  doCheck = false;

  # lmha3 uses common workspace structure
  buildAndTestSubdir = ".";

  meta = with lib; {
    description = "Load Management Hagenholz";
    homepage = "https://git.lluki.me/lmha3";
    license = licenses.mit; # Assuming MIT based on common patterns
    maintainers = [ ];
  };
}
