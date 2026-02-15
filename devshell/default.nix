{ rustToolchain
, cargoArgs
, unitTestArgs
, clangWithLibcxx
, clangStdenv
, pkgs
, ...
}:

let
  cargo-ext = pkgs.callPackage ./cargo-ext.nix { inherit cargoArgs unitTestArgs; };
  # Use clang stdenv to avoid GCC 15 compatibility issues with older RocksDB
  mkShell = pkgs.mkShell.override { stdenv = clangStdenv; };
in
mkShell {
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

    clangWithLibcxx
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
  ] ++ pkgs.lib.optionals clangStdenv.isDarwin [
    iconv
    libiconv
  ];

  buildInputs = with pkgs; [
    jemalloc
  ] ++ pkgs.lib.optionals clangStdenv.hostPlatform.isLinux [
    llvmPackages.libcxx
  ];

  PROTOC = "${pkgs.protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${pkgs.protobuf}/include";

  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

  # Use system jemalloc to avoid tikv-jemalloc-sys build issues with newer glibc
  JEMALLOC_OVERRIDE = "${pkgs.jemalloc}/lib/libjemalloc${if clangStdenv.hostPlatform.isDarwin then ".dylib" else ".so"}";

  # Force cc-rs to use clang with libc++ instead of GCC/libstdc++
  # This avoids GCC 15 compatibility issues with older RocksDB code
  CC = "${clangWithLibcxx}/bin/clang";
  CXX = "${clangWithLibcxx}/bin/clang++";
  # Use -nostdinc++ to disable default C++ include paths, then add libc++ headers
  CXXFLAGS = "-nostdinc++ -isystem ${pkgs.llvmPackages.libcxx.dev}/include/c++/v1 -stdlib=libc++";
  CXXSTDLIB = "c++";

  shellHook = ''
    export NIX_PATH="nixpkgs=${pkgs.path}"
  '';
}
