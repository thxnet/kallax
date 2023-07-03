{ name
, version
, lib
, rustPlatform
, llvmPackages
, protobuf
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
  ];

  doCheck = false;

  PROTOC = "${protobuf}/bin/protoc";
  PROTOC_INCLUDE = "${protobuf}/include";

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
}
