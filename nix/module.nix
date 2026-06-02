{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.lmha3;
in {
  options.services.lmha3 = {
    enable = mkEnableOption "lmha3 load management service";

    package = mkOption {
      type = types.package;
      default = pkgs.callPackage ../default.nix { };
      description = "The lmha3 package to use.";
    };

    port = mkOption {
      type = types.port;
      default = 8000;
      description = "Port to listen on.";
    };

    databaseUrl = mkOption {
      type = types.str;
      example = "postgresql://user:pass@localhost/dbname";
      description = "Database connection string.";
    };

    mqtt = {
      host = mkOption {
        type = types.str;
        default = "solar.lluki.me";
        description = "MQTT broker host.";
      };
      port = mkOption {
        type = types.port;
        default = 1884;
        description = "MQTT broker port.";
      };
      user = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "MQTT username.";
      };
      passwordFile = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Path to a file containing the MQTT password.";
      };
    };

    extraEnvironment = mkOption {
      type = types.attrsOf types.str;
      default = {};
      description = "Extra environment variables for the service.";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.lmha3 = {
      description = "lmha3 Load Management Service";
      after = [ "network.target" "postgresql.service" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Restart = "always";
        User = "lmha3";
        Group = "lmha3";
        DynamicUser = true;
      };

      environment = {
        DATABASE_URL = cfg.databaseUrl;
        MQTT_HOST = cfg.mqtt.host;
        MQTT_PORT = toString cfg.mqtt.port;
      } // optionalAttrs (cfg.mqtt.user != null) {
        MQTT_USER = cfg.mqtt.user;
      } // cfg.extraEnvironment;

      script = ''
        ${optionalString (cfg.mqtt.passwordFile != null) ''
          export MQTT_PASSWORD=$(cat ${cfg.mqtt.passwordFile})
        ''}
        exec ${cfg.package}/bin/server --port ${toString cfg.port}
      '';
    };
  };
}
