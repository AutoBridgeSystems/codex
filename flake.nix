{
  description = "Development Nix flake for OpenAI Adom CLI";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems f;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          adom-rs = pkgs.callPackage ./adom-rs { };
        in
        {
          adom-rs = adom-rs;
          default = adom-rs;
        }
      );
    };
}
