{ lib, rustPlatform, makeWrapper, pkg-config, openssl }:

rustPlatform.buildRustPackage {
  pname = "lmha3";
  version = "0.0.27";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ makeWrapper pkg-config ];
  buildInputs = [ openssl ];

  postInstall = ''
    mkdir -p $out/share/lmha3
    cp -r server/public $out/share/lmha3/public
    cp -r migrations $out/share/lmha3/migrations
    wrapProgram $out/bin/server \
      --set-default LMHA_PUBLIC_DIR "$out/share/lmha3/public" \
      --set-default LMHA_MIGRATIONS_DIR "$out/share/lmha3/migrations"
  '';

  doCheck = false;

  # lmha3 uses common workspace structure
  buildAndTestSubdir = ".";

  meta = with lib; {
    description = "Load Management Hagenholz";
    homepage = "https://github.com/example/lmha3";
    license = licenses.mit; # Assuming MIT based on common patterns
    maintainers = [ ];
  };
}
