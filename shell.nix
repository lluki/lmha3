{ pkgs ? import <nixpkgs> {} }:
(builtins.getFlake (toString ./.)).devShells.${pkgs.system}.default
