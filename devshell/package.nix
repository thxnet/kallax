{ name
, version
, lib
, rustPlatform
, llvmPackages
, protobuf
, jemalloc
, stdenv
, autoPatchelfHook
}:

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
    stdenv.cc.cc.lib
  ];

  doCheck = false;

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

  # Use clang instead of gcc for C/C++ compilation (GCC 15+ has compatibility issues with older RocksDB)
  CC = "${llvmPackages.clang}/bin/clang";
  CXX = "${llvmPackages.clang}/bin/clang++";

  # Use system jemalloc to avoid tikv-jemalloc-sys build issues with newer glibc
  JEMALLOC_OVERRIDE = "${jemalloc}/lib/libjemalloc${if stdenv.hostPlatform.isDarwin then ".dylib" else ".so"}";
}
