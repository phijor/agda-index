{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/unstable";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem
      (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [cargo2nix.overlays.default];
          };

          rustPkgs = pkgs.rustBuilder.makePackageSet {
            rustVersion = "1.66.1";
            packageFun = import ./Cargo.nix;
          };
          agda-index = (rustPkgs.workspace.agda-index {}).bin;
        in rec {
          packages = {
            inherit agda-index;
            default = packages.agda-index;
          };

          devShells.default = rustPkgs.workspaceShell {};
          devShells.bootstrap = pkgs.mkShell {
            buildInputs = [cargo2nix.packages.${system}.cargo2nix];
          };

          apps = {
            agda-index = {
              type = "app";
              program = "${agda-index}/bin/agda-index";
            };
            default = apps.agda-index;
          };
        }
      );
}
