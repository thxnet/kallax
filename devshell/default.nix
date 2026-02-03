{ rustToolchain
, cargoArgs
, unitTestArgs
, pkgs
, ...
}:

let
  cargo-ext = pkgs.callPackage ./cargo-ext.nix { inherit cargoArgs unitTestArgs; };
in
pkgs.mkShell {
  name = "dev-shell";

  nativeBuildInputs = with pkgs; [
    cargo-ext.cargo-build-all
    cargo-ext.cargo-clippy-all
    cargo-ext.cargo-doc-all
    cargo-ext.cargo-nextest-all
    cargo-ext.cargo-test-all
    cargo-ext.cargo-udeps-all
    cargo-ext.cargo-watch-all
    cargo-nextest
    cargo-udeps
    cargo-watch
    rustToolchain

    tokei

    zlib

    llvmPackages.clang
    llvmPackages.libclang

    protobuf

    jq

    hclfmt
    nixpkgs-fmt
    nodePackages.prettier
    shellcheck
    shfmt
    taplo
    treefmt
  ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
    iconv
    libiconv
  ];

  PROTOC = "${pkgs.protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${pkgs.protobuf}/include";

  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

  ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";

  NIX_CFLAGS_COMPILE = "-include stdint.h";

  shellHook = ''
    export NIX_PATH="nixpkgs=${pkgs.path}"
  '';
}
