{ config, lib, pkgs, ... }:
let
  name = "local-search-shortcuts";
  package = pkgs.rustPlatform.buildRustPackage rec {
    pname = name;
    version = "1.2.2";

    src = pkgs.fetchFromGitHub {
      owner = "grantshandy";
      repo = name;
      rev = version;
      hash = "sha256-6uHQ1VkZWtOmY2Q8dgR1if4toMa/czDujaCj65sq498=";
    };

    useFetchCargoVendor = true;
    cargoHash = "sha256-AzhjOoiWWmKzG+/CrN/DUwOERHR1MB2iHzNYM4BJLDA=";
  };

  cfg = config.services.${name};
  tomlFormat = pkgs.formats.toml { };
  configFile = tomlFormat.generate "${name}-config.toml" cfg.settings;
in
{
  options.services.${name} = {
    enable = lib.mkEnableOption name;
    firefoxSearch = lib.mkEnableOption "Set the ${name} provider as the default search engine in Firefox.";
    settings = lib.mkOption {
      type = tomlFormat.type;
      example = lib.literalExpression ''
        {
          port = 9321;
          default = "google";
          engines.homemanager = "https://home-manager-options.extranix.com/?query={s}&release=master";
        }
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.user = {
      enable = true;
      services.${name} = {
        Unit = {
          Description = "Local Search Shortcuts";
          After = [ "network.target" ];
        };
        Path = {
          PathChanged = "${configFile}";
          UnitSec= "1s";
        };
        Service = {
          ExecStart = "${package}/bin/${name}";
          Restart = "always";
        };
        Install.WantedBy = [ "default.target" ];
      };
    };

    xdg.configFile."${name}/config.toml".source = configFile;

    programs.firefox.profiles.default.search =
      let
        port = toString cfg.settings.port or 9321;
        default-engine = cfg.settings.default;
        errMsg = "services.${name}.settings.default must be set with services.${name}.settings.firefoxSearch is enabled";
      in
        lib.mkIf (cfg.firefoxSearch && lib.asserts.assertMsg (lib.attrsets.hasAttr "default" cfg.settings) errMsg)
      {
        force = true;
        default = default-engine;
        privateDefault = default-engine;
        engines.${default-engine}.urls = [{
          name = default-engine;
          template = "http://localhost:${port}/?q={searchTerms}";
        }];
      };
  };
}
