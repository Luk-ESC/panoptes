{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    flake-utils,
    naersk,
    nixpkgs,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};
      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          src = ./.;

          postInstall = ''
            #wrapProgram $out/bin/panoptes --SET WHAT bruh
          '';
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [rustc cargo];
        };

        nixosModules = {
          panoptes-service = {
            config,
            pkgs,
            lib,
            ...
          }: {
            options = {};
            config = {
              systemd.services.panoptes = {
                description = "Panoptes Background Service";
                after = ["network.target"];
                wantedBy = ["multi-user.target"];

                serviceConfig = {
                  ExecStart = "${self.defaultPackage.${pkgs.system}}/bin/panoptes";
                  Restart = "on-failure";
                };
              };
            };
          };
        };
      }
    );
}
