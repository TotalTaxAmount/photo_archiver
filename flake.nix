{
  description = "Photo Archiver flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
  let
    pkgs = nixpkgs.legacyPackages.${system};
    pname = "photo-archiver";
    version = "0.0.0-DEV";

    node-modules = pkgs.mkYarnPackage {
      name = "${pname}-node-modules";
      version = version;
      src = ./archive-frontend;
    };

    frontend = pkgs.stdenv.mkDerivation {
      name = "${pname}-frontend";
      version = version;

      src = ./archive-frontend;

      buildInputs = [pkgs.yarn node-modules pkgs.nodejs_23];
      buildPhase = ''
        ln -s ${node-modules}/libexec/archive-frontend/node_modules node_modules
        yarn build
      '';

      installPhase = ''
        mkdir -p $out/share/frontend
        mv build/* $out/share/frontend
      '';
    };

    backend = pkgs.rustPlatform.buildRustPackage {
      pname = "${pname}-backend";
      version = version;

      src = ./.;

      cargoLock = {
        lockFile = ./Cargo.lock;
        outputHashes = {
          "webrs-0.2.0" = "sha256-PvlMNbUOVzXT3LwvOW9nio7NP87+1gwe1A9C/2pA6jo=";
        };
      };

      nativeBuildInputs = with pkgs; [ pkg-config openssl yarn ];
      buildInputs = with pkgs; [ openssl ];        
    };
  in
  { 
    packages = {
      default = pkgs.stdenv.mkDerivation {
        name = pname;
        version = version;

        buildInputs = [backend frontend];
        phases = [ "installPhase" ];
        installPhase = ''
          mkdir -p $out/bin $out/share
          cp ${backend}/bin/photo_archiver $out/bin/${pname}
          cp -r ${frontend}/share/ $out/share/
        '';
      };
      frontend = frontend;
      backend = backend;
    };

    devShell = pkgs.mkShell {
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
  });
}
