{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [ 
        cargo rustc rustfmt clippy pkg-config gcc openssl zlib 
        wayland libxkbcommon
        xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi
      ];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [ 
        wayland libxkbcommon libGL 
        xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi
      ]);
    };
  };
}
