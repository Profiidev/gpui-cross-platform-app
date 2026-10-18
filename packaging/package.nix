{
  lib,
  rustPlatform,
  pkg-config,
  fontconfig,
  freetype,
  libxkbcommon,
  libxcb,
  wayland,
  vulkan-loader,
}:

let
  # gpui dlopens these
  gpuiLibs = [
    wayland
    vulkan-loader
    libxkbcommon
  ];
in

rustPlatform.buildRustPackage {
  pname = "{{project-name}}";
  version = (lib.importTOML ../Cargo.toml).package.version;

  src = ./..;

  cargoLock = {
    lockFile = ../Cargo.lock;
    # gpui-mobile is only used on Android/iOS, but the lockfile still vendors it
    outputHashes = {
      "gpui-pre-mobile-0.1.0" = "sha256-PWxp1t8Q6cZ0TQKxaS67zS2hNFyJcsmWbgnJjcPzZQM=";
    };
  };

  cargoBuildFlags = [
    "--bin"
    "{{project-name}}"
  ];

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [
    fontconfig
    freetype
    libxkbcommon
    libxcb
  ];

  # the gpui tests may open windows, CI runs them (tests.yml)
  doCheck = false;

  postInstall = ''
    install -Dm644 packaging/icons/32x32.png $out/share/icons/hicolor/32x32/apps/{{project-name}}.png
    install -Dm644 packaging/icons/128x128.png $out/share/icons/hicolor/128x128/apps/{{project-name}}.png
    install -Dm644 packaging/icons/128x128@2x.png $out/share/icons/hicolor/256x256/apps/{{project-name}}.png
    install -Dm644 packaging/app.desktop $out/share/applications/{{project-name}}.desktop
    substituteInPlace $out/share/applications/{{project-name}}.desktop \
      --replace-fail "{% raw %}{{categories}}{% endraw %}" "Utility;" \
      --replace-fail "{% raw %}{{exec}}{% endraw %}" "$out/bin/{{project-name}}" \
      --replace-fail "{% raw %}{{icon}}{% endraw %}" "{{project-name}}" \
      --replace-fail "{% raw %}{{name}}{% endraw %}" "{{app_name}}"
    sed -i '/{% raw %}{{{% endraw %}/d' $out/share/applications/{{project-name}}.desktop
  '';

  postFixup = ''
    patchelf --add-rpath ${lib.makeLibraryPath gpuiLibs} $out/bin/{{project-name}}
  '';

  meta = {
    description = "{{app_name}}";
    homepage = "https://github.com/Profiidev/{{project-name}}";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "{{project-name}}";
  };
}
