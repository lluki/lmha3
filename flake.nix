{
  description = "Load Management Hagenholz (lmha3)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.callPackage ./default.nix { };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            postgresql
            pkg-config
            openssl
          ];
          shellHook = ''
            export DATABASE_URL="host=/var/run/postgresql dbname=lmha3 user=$(whoami)"
          '';
        };
      }
    ) // {
      nixosModules.default = import ./nix/module.nix;
    };
}
