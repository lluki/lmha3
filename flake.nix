{
  description = "Load Management Hagenholz (lmha3)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    flake-utils.url = "github:numtide/flake-utils";
    openspec.url = "github:Fission-AI/OpenSpec";
  };

  outputs = { self, nixpkgs, flake-utils, openspec }:
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
            gemini-cli
            openspec.packages.${system}.default
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
