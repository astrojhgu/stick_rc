# default.nix
with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "mpi_rust"; # Probably put a more meaningful name here
    buildInputs = [
        pkg-config
        udev.dev
        openssl.dev
        SDL2.dev
    ];
    hardeningDisable = [ "all" ];
    
}
