{ pkgs ? import <nixpkgs> {} }:
let
  hostPkgs = pkgs.pkgsCross.aarch64-multiplatform;
in pkgs.mkShell {
  shellHook = ''
    echo "objdump all targets: ${pkgs.binutils-unwrapped-all-targets.out}/bin/objdump"

    # https://nix.dev/tutorials/cross-compilation
    # pkgsCross is not on cache.nixos.org, so we would need to build binutils
    # echo "aarch64 emulator: $[delete me]{hostPkgs.stdenv.hostPlatform.emulator hostPkgs.buildPackages}"
    # echo "aarch64 objdump: $[delete me]{pkgs.pkgsCross.aarch64-multiplatform.binutils.out}/bin/objdump"

    export PATH=${pkgs.binutils-unwrapped-all-targets.out}/bin:$PATH
  '';

  buildInputs = [
    # Makefile
    pkgs.gnumake
    pkgs.curl
    pkgs.gzip

    pkgs.rustup
    pkgs.qemu
  ];
}
