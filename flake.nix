{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      fenix,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustToolchain =
          with fenix.packages.${system};
          combine [
            (complete.withComponents [
              # use rust nightly
              "rustc"
              "cargo"
              "rustfmt"
              "clippy"
              "rust-src"
              "rust-analyzer"
              "miri"
            ])
          ];
      in
      {
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              rustToolchain
            ];
          };
      }
    );
}
