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
  pname = "template";
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
    "template"
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
    install -Dm644 packaging/icons/32x32.png $out/share/icons/hicolor/32x32/apps/template.png
    install -Dm644 packaging/icons/128x128.png $out/share/icons/hicolor/128x128/apps/template.png
    install -Dm644 packaging/icons/128x128@2x.png $out/share/icons/hicolor/256x256/apps/template.png
    install -Dm644 packaging/template.desktop $out/share/applications/template.desktop
    substituteInPlace $out/share/applications/template.desktop \
      --replace-fail "{{categories}}" "Utility;" \
      --replace-fail "{{exec}}" "$out/bin/template" \
      --replace-fail "{{icon}}" "template" \
      --replace-fail "{{name}}" "Template"
    sed -i '/{{/d' $out/share/applications/template.desktop
  '';

  postFixup = ''
    patchelf --add-rpath ${lib.makeLibraryPath gpuiLibs} $out/bin/template
  '';

  meta = {
    description = "GPUI Kit app template for desktop, Android and iOS";
    homepage = "https://github.com/Profiidev/template";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "template";
  };
}
