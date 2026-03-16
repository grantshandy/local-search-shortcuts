# home-manager module that automatically enables local-search-shortcuts.
# Works as of home-manager/nixos 25.11
{
  config,
  lib,
  pkgs,
  ...
}: let
  name = "local-search-shortcuts";
  package = pkgs.rustPlatform.buildRustPackage rec {
    pname = name;
    version = "1.2.3";

    src = pkgs.fetchFromGitHub {
      owner = "grantshandy";
      repo = name;
      rev = version;
      hash = "sha256-ylIeBGKMtR9Lj2gk1Pkzs/zi4NLjI5sJyYgmTVXjkrk=";
    };

    cargoHash = "sha256-Kxrucvdy/ECkQrXmXiDct6/bO1yF76JZc/OSmvginVU=";
    meta.mainProgram = name;
  };

  cfg = config.services.${name};
in {
  options.services.${name} = {
    enable = lib.mkEnableOption name;

    firefoxSearch = lib.mkEnableOption "Set the ${name} provider as the default search engine in Firefox.";

    port = lib.mkOption {
      type = lib.types.port;
      default = 9321;
    };

    broadcast = lib.mkOption {
      type = lib.types.bool;
      default = false;
    };

    engines = lib.mkOption {
      type = lib.types.attrs;
      default = {};
    };

    default = lib.mkOption {
      type = lib.types.str;
      default = "ddg";
    };

    displayName = lib.mkOption {
      type = lib.types.str;
      default = "lss";
    };
  };

  config = let
    configFile = (pkgs.formats.toml {}).generate "${name}-config.toml" {
      inherit (cfg) port engines default broadcast;
    };
  in
    lib.mkIf cfg.enable {
      xdg.configFile."${name}/config.toml".source = configFile;

      systemd.user.services.${name} = {
        Unit = {
          Description = "Local Search Shortcuts";
          After = ["network.target"];
          X-Restart-Triggers = [
            configFile
          ];
        };
        Service = {
          ExecStart = lib.getExe package;
          Restart = "always";
        };
        Install.WantedBy = ["default.target"];
      };

      programs.firefox.profiles.default.search =
        lib.mkIf cfg.firefoxSearch
        {
          force = true;
          default = cfg.displayName;
          privateDefault = cfg.displayName;
          engines.${cfg.displayName}.urls = [
            {
              template = "http://localhost:${toString cfg.port}/";
              params = [
                {
                  name = "q";
                  value = "{searchTerms}";
                }
              ];
            }
          ];
        };
    };
}
