{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    inputs:
    with inputs;
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        cargoNix = pkgs.callPackage ./Cargo.nix { inherit pkgs; };
        agda-index = cargoNix.rootCrate.build;
      in
      rec {
        packages = {
          inherit agda-index;
          default = packages.agda-index;
        };

        devShells.default = pkgs.mkShell { inputsFrom = [ packages.default ]; };
        devShells.bootstrap = pkgs.mkShell { buildInputs = [ pkgs.crate2nix ]; };

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
