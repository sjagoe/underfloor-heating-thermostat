{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
  };

  outputs = {
    self,
    nixpkgs
  }: let
    pkgs = import nixpkgs {
      system = "x86_64-linux";
    };
    fhs = pkgs.buildFHSUserEnv {
      name = "fhs-shell";
      targetPkgs = pkgs: with pkgs; [
        gcc

        pkg-config
        libclang.lib
        gnumake
        cmake
        ninja

        git
        wget

        espflash
        python3
        python3Packages.pip
        python3Packages.virtualenv
      ];
    };
  in {
    devShells.${pkgs.system}.default = fhs.env;
  };
}
