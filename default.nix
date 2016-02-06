{ pkgs ? (import <nixpkgs> {}) }:

let
  env = with pkgs.rustStable; [
    rustc
    cargo
    pkgs.llvmPackages.lldb
    pkgs.libgit2
    pkgs.libssh2
    pkgs.openssl
    pkgs.pkgconfig
  ];
in

pkgs.stdenv.mkDerivation rec {
    name = "imag";
    src = ./.;
    version = "0.0.0";

    buildInputs = [ env ];

}

