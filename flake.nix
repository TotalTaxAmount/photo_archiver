{
  description = "Photo Archiver flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux"; 
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "photo_archiver";
        version = "0.0.1";

        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
          outputHashes = {
            "webrs-0.2.0" = "sha256-uh/pQGV+FVnDKpjzjzB2KJzqkMeITIhyfZaJUQVJZp0=";
          };
        };

        nativeBuildInputs = with pkgs; [ pkg-config yarn ];
        buildInputs = with pkgs; [ openssl ];
      };

      devShell.${system} = pkgs.mkShell {
        buildInputs = with pkgs; [ 
          rustup 
          cargo 
          pkg-config 
          openssl 
          nodejs_23
          yarn
          sea-orm-cli
        ];
      };
    };
}

