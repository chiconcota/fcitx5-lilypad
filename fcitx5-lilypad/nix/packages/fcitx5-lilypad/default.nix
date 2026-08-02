{
  lib,
  stdenv,
  acl,
  buildGoModule,
  cmake,
  fcitx5,
  fetchFromGitHub,
  gettext,
  go,
  hicolor-icon-theme,
  kdePackages,
  libinput,
  libx11,
  pkg-config,
  python3,
  qt6,
  udev,
}:
stdenv.mkDerivation rec {
  pname = "fcitx5-lilypad";
  version = "3.4.0";

  src = fetchFromGitHub {
    owner = "LilypadInputMethod";
    repo = "fcitx5-lilypad";
    rev = "v${version}";
    fetchSubmodules = true;
    hash = "sha256-MN83U0/o+vDGCxpYgFxfXAf+Iw59OaXyB7770ppLmEQ=";
  };

  nativeBuildInputs = [
    cmake
    kdePackages.extra-cmake-modules
    gettext
    go
    hicolor-icon-theme
    pkg-config
    qt6.wrapQtAppsHook
  ];

  buildInputs = [
    acl
    fcitx5
    libinput
    libx11
    (python3.withPackages (
      ps: with ps; [
        pyqt6
        dbus-python
        qtpy
      ]
    ))
    qt6.qtbase
    udev
  ];

  vendorDir =
    (buildGoModule {
      pname = "fcitx5-lilypad-go-modules";
      inherit version src;
      modRoot = "bamboo";
      vendorHash = "sha256-HjVMGil4bNMTFifxFYtHELdkeKhrumHGrde4msbxvJc=";
    }).goModules;

  preConfigure = ''
    export GOCACHE=$TMPDIR/go-cache
    export GOPATH=$TMPDIR/go

    rm -rf bamboo/vendor
    cp -r $vendorDir bamboo/vendor
  '';

  cmakeFlags = [
    "-DGO_FLAGS=-mod=vendor"
  ];

  # change checking exe_path logic to make it work on NixOS since executable files on NixOS are not located in /usr/bin
  postPatch = ''
    substituteInPlace src/lilypad-monitor.cpp \
      --replace-fail 'strcmp(exe_path, "/usr/bin/fcitx5-lilypad-server") == 0' \
                '(strncmp(exe_path, "/nix/store/", 11) == 0 && strlen(exe_path) >= 24 && strcmp(exe_path + strlen(exe_path) - 24, "/bin/fcitx5-lilypad-server") == 0)'
    substituteInPlace server/lilypad-server.cpp \
      --replace-fail 'strcmp(exe_path, "/usr/bin/fcitx5") == 0' \
                '(strncmp(exe_path, "/nix/store/", 11) == 0 && strlen(exe_path) >= 11 && strcmp(exe_path + strlen(exe_path) - 11, "/bin/fcitx5") == 0)'
  '';

  postInstall = ''
    substituteInPlace $out/lib/udev/rules.d/99-lilypad.rules \
      --replace-fail "/usr/bin/setfacl" "${acl}/bin/setfacl"
    substituteInPlace $out/lib/systemd/system/fcitx5-lilypad-server@.service \
      --replace-fail "/usr/bin/setfacl" "${acl}/bin/setfacl"
    substituteInPlace $out/lib/systemd/system/fcitx5-lilypad-server@.service \
      --replace-fail "/usr/bin/fcitx5-lilypad-server" "$out/bin/fcitx5-lilypad-server"
  '';

  postFixup = ''
    patchShebangs $out/share/fcitx5-lilypad/settings-gui
    wrapQtApp $out/bin/fcitx5-lilypad-settings
  '';

  meta = with lib; {
    description = "Fcitx5 Lilypad input method for Vietnamese typing";
    license = licenses.gpl3;
    platforms = platforms.linux;
  };
}
