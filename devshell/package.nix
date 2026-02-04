{ name
, version
, lib
, rustPlatform
, llvmPackages
, protobuf
, jemalloc
, autoPatchelfHook
, clangWithLibcxx
, clangStdenv
}:

# Use clangStdenv to avoid GCC 15 compatibility issues with older RocksDB
# The stdenv parameter overrides the derivation's default stdenv
rustPlatform.buildRustPackage {
  pname = name;
  inherit version;

  # Override stdenv to use clang instead of gcc
  stdenv = clangStdenv;

  src = lib.cleanSource ./..;

  cargoLock = {
    outputHashes = {
      "sc-allocator-4.1.0-dev" = "sha256-0/bW4cY+uVIsVjzyqqom3DtOWYQQsCJftBLTjsOJ6DQ=";
    };
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [
    llvmPackages.clang
    llvmPackages.libclang
  ] ++ lib.optionals clangStdenv.hostPlatform.isLinux [
    autoPatchelfHook
  ];

  buildInputs = [
    jemalloc
  ] ++ lib.optionals clangStdenv.hostPlatform.isLinux [
    llvmPackages.libcxx
  ];

  doCheck = false;

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

  # Use system jemalloc to avoid tikv-jemalloc-sys build issues with newer glibc
  JEMALLOC_OVERRIDE = "${jemalloc}/lib/libjemalloc${if clangStdenv.hostPlatform.isDarwin then ".dylib" else ".so"}";

  # Force cc-rs to use our custom clang with libc++ instead of GCC/libstdc++
  CC = "${clangWithLibcxx}/bin/clang";
  CXX = "${clangWithLibcxx}/bin/clang++";
  CXXSTDLIB = "c++";
}
