{
  description = "A web forum";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "x86_64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "webeng";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            buildInputs = [ self.packages.${system}.frontend ];
            NIX_BUILD = 1;
          };
          frontend = pkgs.buildNpmPackage {
            pname = "frontend";
            version = "0.1.0";
            src = ./frontend;
            npmDepsHash = "sha256-0iV4n92oSnBJ/xVkhTyurqzIVMG5dSDmavnHwrbB0jU=";
            buildInputs = [ pkgs.nodejs ];

          };
        }
      );
    };
}
