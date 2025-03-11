{
  description = "still alive";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    oxalica.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, oxalica }:
    with flake-utils.lib;
    eachSystem allSystems (system:
      let
        pkgs = nixpkgs.legacyPackages.${system}.extend oxalica.overlays.default;
      in rec {

        packages = {
          still-alive = let
            rustPlatform = pkgs.makeRustPlatform {
              cargo = pkgs.rust-bin.stable.latest.minimal;
              rustc = pkgs.rust-bin.stable.latest.minimal;
            };
          in rustPlatform.buildRustPackage rec {
            name = "still-alive";
            src = self;

            nativeBuildInputs = with pkgs; [ pkg-config makeWrapper ];
            buildInputs = with pkgs; [ alsa-lib.dev ];

            postFixup = ''
              mv $out/bin/still-alive $out/bin/.still-alive-wrapped
              echo "#! ${pkgs.bash}/bin/bash" > $out/bin/still-alive
              echo "exec ${pkgs.kitty}/bin/kitty -c /dev/null $out/bin/.still-alive-wrapped & disown" >> $out/bin/still-alive
              chmod +x $out/bin/still-alive
            '';

            cargoLock = { lockFile = ./Cargo.lock; };
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          };
        };
        defaultPackage = packages.still-alive;
        formatter = pkgs.nixfmt;
      });
}
