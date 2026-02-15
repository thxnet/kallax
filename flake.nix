{
  description = "Kallax - Utilities for setting up and managing Substrate-based blockchains";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, crane }:
    let
      name = "kallax";
      version = "0.3.6";
    in
    (flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              self.overlays.default
              fenix.overlays.default
            ];
          };

          rustToolchain = fenix.packages.${system}.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-vra6TkHITpwRyA5oBKAHSX0Mi6CBDNQD+ryPSpxFsfg=";
          };

          # Create a clang wrapper with libc++ as the default stdlib
          clangWithLibcxx = pkgs.wrapCCWith {
            cc = pkgs.llvmPackages.clang-unwrapped;
            bintools = pkgs.llvmPackages.bintools;
            extraBuildCommands = ''
              # Set flags for C++ compilation specifically
              echo "-stdlib=libc++" >> $out/nix-support/cc-cxxflags
              echo "-nostdinc++" >> $out/nix-support/cc-cxxflags
              echo "-isystem ${pkgs.llvmPackages.libcxx.dev}/include/c++/v1" >> $out/nix-support/cc-cxxflags
              # Also add -lc++ to link against libc++
              echo "-lc++" >> $out/nix-support/cc-ldflags
            '';
          };

          # Use a custom stdenv with our clang+libc++ wrapper
          # This ensures cc-rs uses our compiler regardless of environment variables
          clangStdenv = pkgs.overrideCC pkgs.llvmPackages.stdenv clangWithLibcxx;

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
            stdenv = clangStdenv;
          };

          craneLib = ((crane.mkLib pkgs).overrideToolchain rustToolchain).overrideScope (final: prev: {
            stdenv = clangStdenv;
          });

          cargoArgs = [
            "--workspace"
            "--bins"
            "--examples"
            "--tests"
            "--benches"
            "--all-targets"
          ];

          unitTestArgs = [
            "--workspace"
          ];

          # Custom source filter that includes .proto and .json files
          src = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = path: type:
              # Include .proto files for protobuf compilation
              (pkgs.lib.hasSuffix ".proto" path) ||
              # Include .json files for chain-spec include_bytes!
              (pkgs.lib.hasSuffix ".json" path) ||
              # Use crane's default filter for everything else
              (craneLib.filterCargoSources path type);
          };

          jemallocLib =
            if pkgs.stdenv.hostPlatform.isDarwin
            then "${pkgs.jemalloc}/lib/libjemalloc.dylib"
            else "${pkgs.jemalloc}/lib/libjemalloc.so";

          commonArgs = {
            inherit src;

            # Use clang stdenv for the build
            stdenv = clangStdenv;

            nativeBuildInputs = with pkgs; [
              # Use clangWithLibcxx to ensure libc++ headers are available
              clangWithLibcxx
              llvmPackages.libclang
            ] ++ pkgs.lib.optionals clangStdenv.hostPlatform.isLinux [
              pkgs.autoPatchelfHook
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
            JEMALLOC_OVERRIDE = jemallocLib;

            # Force cc-rs to use our custom clang with libc++ instead of GCC/libstdc++
            CC = "${clangWithLibcxx}/bin/clang";
            CXX = "${clangWithLibcxx}/bin/clang++";
            CXXSTDLIB = "c++";

            # Force libc++ headers via CXXFLAGS (cc-rs may not use wrapper's cc-cxxflags)
            CXXFLAGS = "-nostdinc++ -isystem ${pkgs.llvmPackages.libcxx.dev}/include/c++/v1 -stdlib=libc++";
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        in
        {
          formatter = pkgs.treefmt;

          devShells.default = pkgs.callPackage ./devshell {
            inherit rustToolchain cargoArgs unitTestArgs clangWithLibcxx clangStdenv;
          };

          packages = rec {
            default = kallax;
            # Use crane instead of rustPlatform.buildRustPackage to ensure
            # consistent stdenv (clangStdenv) is used for cc-rs builds
            kallax = craneLib.buildPackage (commonArgs // {
              inherit cargoArtifacts;
              pname = name;
              inherit version;
              doCheck = false;
            });
            container = pkgs.callPackage ./devshell/container.nix {
              inherit name version kallax;
            };
          };

          checks = {
            format = pkgs.callPackage ./devshell/format.nix { };

            rust-build = craneLib.cargoBuild (commonArgs // {
              inherit cargoArtifacts;
            });
            rust-format = craneLib.cargoFmt { inherit src; };
            rust-clippy = craneLib.cargoClippy (commonArgs // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = pkgs.lib.strings.concatMapStrings (x: x + " ") cargoArgs;
            });
            rust-nextest = craneLib.cargoNextest (commonArgs // {
              inherit cargoArtifacts;
              cargoExtraArgs = "--workspace --all-targets";
              partitions = 1;
              partitionType = "count";
            });
          };
        })) // {
      overlays.default = final: prev: { };
    };
}
