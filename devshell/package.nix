{ name
, version
, lib
, rustPlatform
, llvmPackages
, protobuf
, rocksdb
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
    rocksdb
    jemalloc
  ] ++ lib.optionals stdenv.hostPlatform.isLinux [
    stdenv.cc.cc.lib
  ];

  doCheck = false;

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

  ROCKSDB_LIB_DIR = "${rocksdb}/lib";
  ROCKSDB_INCLUDE_DIR = "${rocksdb}/include";

  JEMALLOC_OVERRIDE = "${jemalloc}/lib/libjemalloc${if stdenv.hostPlatform.isDarwin then ".dylib" else ".so"}";
}
