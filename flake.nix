{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk' = pkgs.callPackage naersk { };
      in
      {
        packages = rec {
          # For `nix build .#ip_rs`
          ip_rs = naersk'.buildPackage {
            src = ./.;
          };

          # For `nix build .#ociImage` and then `podman image load ./result`
          ociImage = pkgs.dockerTools.buildLayeredImage {
            name = "registry.fly.io/iprs";
            tag = "latest";

            contents = [ pkgs.coreutils ip_rs ];
            config = {
              Cmd = [ "${ip_rs}/bin/ip-rs" ];
            };
          };
        };

        # For `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cargo
            clippy
            flyctl
            rust-analyzer
            rustc
            rustfmt
          ];
        };
      });
}
