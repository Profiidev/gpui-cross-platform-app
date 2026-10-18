{ pkgs, ... }:

{
  packages = with pkgs; [
    cargo-ndk
    just
  ];

  languages = {
    java.jdk.package = pkgs.jdk17;
  };

  android = {
    enable = true;
    platforms.version = [ "34" ];
    buildTools.version = [ "36.0.0" ];
    ndk.enable = true;
  };
}
