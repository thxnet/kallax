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
      version = "0.3.5";
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
              echo "-stdlib=libc++" >> $out/nix-support/cc-cflags
              echo "-nostdinc++" >> $out/nix-support/cc-cflags
              echo "-isystem ${pkgs.llvmPackages.libcxx.dev}/include/c++/v1" >> $out/nix-support/cc-cflags
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

          src = craneLib.cleanCargoSource (craneLib.path ./.);

          jemallocLib =
            if pkgs.stdenv.hostPlatform.isDarwin
            then "${pkgs.jemalloc}/lib/libjemalloc.dylib"
            else "${pkgs.jemalloc}/lib/libjemalloc.so";

          commonArgs = {
            inherit src;

            # Use clang stdenv for the build
            stdenv = clangStdenv;

            nativeBuildInputs = with pkgs; [
              llvmPackages.clang
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
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        in
        {
          formatter = pkgs.treefmt;

          devShells.default = pkgs.callPackage ./devshell {
            inherit rustToolchain cargoArgs unitTestArgs;
          };

          packages = rec {
            default = kallax;
            kallax = pkgs.callPackage ./devshell/package.nix {
              inherit name version rustPlatform;
            };
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
              partitions = 1;
              partitionType = "count";
            });
          };
        })) // {
      overlays.default = final: prev: { };
    };
}
