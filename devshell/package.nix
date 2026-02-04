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

  # Put clangWithLibcxx first to ensure it's found before any GCC in PATH
  nativeBuildInputs = [
    clangWithLibcxx
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

  # Set target-specific environment variables for cc-rs
  # cc-rs checks TARGET_{triple}_CC and TARGET_{triple}_CXX before CC/CXX
  TARGET_CC = "${clangWithLibcxx}/bin/clang";
  TARGET_CXX = "${clangWithLibcxx}/bin/clang++";

  # Use preConfigure to create Cargo config and override environment
  preConfigure = ''
    export PATH="${clangWithLibcxx}/bin:$PATH"
    export NIX_CC="${clangWithLibcxx}"
    export CC="${clangWithLibcxx}/bin/clang"
    export CXX="${clangWithLibcxx}/bin/clang++"
    export TARGET_CC="${clangWithLibcxx}/bin/clang"
    export TARGET_CXX="${clangWithLibcxx}/bin/clang++"

    # Create .cargo/config.toml to tell cc-rs to use our clang
    mkdir -p .cargo
    cat > .cargo/config.toml << EOF
    [target.x86_64-unknown-linux-gnu]
    linker = "${clangWithLibcxx}/bin/clang"

    [env]
    CC = "${clangWithLibcxx}/bin/clang"
    CXX = "${clangWithLibcxx}/bin/clang++"
    EOF
  '';
}
