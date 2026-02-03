{ name
, version
, lib
, rustPlatform
, llvmPackages
, protobuf
, jemalloc
, autoPatchelfHook
}:

# Note: rustPlatform is expected to be built with llvmPackages.stdenv (clang)
# to avoid GCC 15 compatibility issues with older RocksDB
let
  stdenv = llvmPackages.stdenv;
in
rustPlatform.buildRustPackage {
  pname = name;
  inherit version;

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
  ] ++ lib.optionals stdenv.hostPlatform.isLinux [
    autoPatchelfHook
  ];

  buildInputs = [
    jemalloc
  ] ++ lib.optionals stdenv.hostPlatform.isLinux [
    llvmPackages.libcxx
  ];

  doCheck = false;

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

  # Use system jemalloc to avoid tikv-jemalloc-sys build issues with newer glibc
  JEMALLOC_OVERRIDE = "${jemalloc}/lib/libjemalloc${if stdenv.hostPlatform.isDarwin then ".dylib" else ".so"}";
}
