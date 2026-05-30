{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
    crane.url = "github:ipetkov/crane";
  };
  outputs = { nixpkgs, flake-utils, rust-overlay, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        craneLib = (crane.mkLib pkgs).overrideToolchain (p:
          p.pkgsBuildHost.rust-bin.fromRustupToolchainFile
          ./rust-toolchain.toml);

        ip-info = let
          # Askama templates and CSS live outside Cargo's source tree, so the
          # default Cargo source filter would strip them from the build sandbox.
          assetFilter = path: _type:
            builtins.match ".*(css|html)$" path != null;
          assetOrCargo = path: type:
            (assetFilter path type) || (craneLib.filterCargoSources path type);
        in craneLib.buildPackage {
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = assetOrCargo;
            name = "source";
          };
        };
      in {
        devShells.default = craneLib.devShell { inputsFrom = [ ip-info ]; };
        packages.default = ip-info;
      });
}
