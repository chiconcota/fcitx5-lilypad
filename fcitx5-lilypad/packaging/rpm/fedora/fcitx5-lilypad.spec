Name:           fcitx5-lilypad
Version:        3.4.0
Release:        1
Summary:        Vietnamese input method for fcitx5
License:        GPL-3.0-or-later
URL:            https://github.com/LilypadInputMethod/fcitx5-lilypad
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cmake
BuildRequires:  extra-cmake-modules
BuildRequires:  gcc-c++
BuildRequires:  gettext-devel
BuildRequires:  glibc-devel
BuildRequires:  cmake(Fcitx5Core)
BuildRequires:  libinput-devel
BuildRequires:  systemd-rpm-macros
BuildRequires:  pkgconfig(libudev)
BuildRequires:  libX11-devel

BuildRequires:  golang
BuildRequires:  python3
BuildRequires:  libgudev-devel

%{?systemd_requires}
Requires:       fcitx5-data
Requires:       fcitx5
Requires:       python3-QtPy
Requires:       (python3-pyqt6 or python3-pyside6)
Requires:       python3-dbus
Requires:       hicolor-icon-theme
Requires:       acl

%description
Vietnamese input method for fcitx5

%prep
%setup -q

%build
%cmake
%cmake_build

%install
%cmake_install
%find_lang %{name}

%files -f %{name}.lang
%defattr(-,root,root,-)
%dir %{_datadir}/licenses/%{name}
%license %{_datadir}/licenses/%{name}/GPL-3.0-or-later.txt
%license %{_datadir}/licenses/%{name}/LGPL-2.1-or-later.txt
%{_bindir}/fcitx5-lilypad-server
%{_bindir}/fcitx5-lilypad-settings

%dir %{_libdir}/fcitx5
%{_libdir}/fcitx5/liblilypad.so

%{_prefix}/lib/modules-load.d/fcitx5-lilypad.conf
%{_unitdir}/fcitx5-lilypad-server@.service
%{_prefix}/lib/sysusers.d/lilypad.conf
%{_prefix}/lib/udev/rules.d/99-lilypad.rules

%{_datadir}/fcitx5/addon/lilypad.conf
%{_datadir}/fcitx5/inputmethod/lilypad.conf

%dir %{_datadir}/fcitx5/lilypad
%{_datadir}/fcitx5/lilypad/vietnamese.cm.dict

%{_datadir}/fcitx5-lilypad/settings-gui/
%{_datadir}/applications/org.fcitx.Fcitx5.Addon.Lilypad.Settings.desktop

%{_datadir}/icons/hicolor/scalable/apps/fcitx-lilypad.svg
%{_datadir}/icons/hicolor/scalable/apps/org.fcitx.Fcitx5.fcitx-lilypad.svg
%{_datadir}/icons/hicolor/scalable/apps/fcitx-lilypad-off.svg
%{_datadir}/icons/hicolor/scalable/apps/org.fcitx.Fcitx5.fcitx-lilypad-off.svg
%{_datadir}/icons/hicolor/scalable/apps/fcitx-lilypad-emoji.svg
%{_datadir}/icons/hicolor/scalable/apps/org.fcitx.Fcitx5.fcitx-lilypad-emoji.svg
%{_datadir}/icons/hicolor/scalable/apps/fcitx-lilypad-emoji-default.svg
%{_datadir}/icons/hicolor/scalable/apps/fcitx-lilypad-default.svg
%{_datadir}/icons/hicolor/scalable/apps/fcitx-lilypad-off-default.svg
%{_datadir}/icons/hicolor/scalable/apps/fcitx-lilypad-emoji-default-black.svg
%{_datadir}/icons/hicolor/scalable/apps/fcitx-lilypad-default-black.svg
%{_datadir}/icons/hicolor/scalable/apps/fcitx-lilypad-off-default-black.svg

%{_datadir}/icons/breeze/status/22/fcitx-lilypad-default.svg
%{_datadir}/icons/breeze/status/22/fcitx-lilypad-off-default.svg
%{_datadir}/icons/breeze/status/22/fcitx-lilypad-emoji-default.svg
%{_datadir}/icons/breeze/status/22/fcitx-lilypad-default-black.svg
%{_datadir}/icons/breeze/status/22/fcitx-lilypad-off-default-black.svg
%{_datadir}/icons/breeze/status/22/fcitx-lilypad-emoji-default-black.svg
%{_datadir}/icons/breeze/status/22/fcitx-lilypad.svg
%{_datadir}/icons/breeze/status/22/fcitx-lilypad-off.svg
%{_datadir}/icons/breeze/status/22/fcitx-lilypad-emoji.svg

%{_datadir}/icons/breeze/status/24/fcitx-lilypad-default.svg
%{_datadir}/icons/breeze/status/24/fcitx-lilypad-off-default.svg
%{_datadir}/icons/breeze/status/24/fcitx-lilypad-emoji-default.svg
%{_datadir}/icons/breeze/status/24/fcitx-lilypad-default-black.svg
%{_datadir}/icons/breeze/status/24/fcitx-lilypad-off-default-black.svg
%{_datadir}/icons/breeze/status/24/fcitx-lilypad-emoji-default-black.svg
%{_datadir}/icons/breeze/status/24/fcitx-lilypad.svg
%{_datadir}/icons/breeze/status/24/fcitx-lilypad-off.svg
%{_datadir}/icons/breeze/status/24/fcitx-lilypad-emoji.svg

%{_datadir}/icons/breeze-dark/status/22/fcitx-lilypad-default.svg
%{_datadir}/icons/breeze-dark/status/22/fcitx-lilypad-off-default.svg
%{_datadir}/icons/breeze-dark/status/22/fcitx-lilypad-emoji-default.svg
%{_datadir}/icons/breeze-dark/status/22/fcitx-lilypad-default-black.svg
%{_datadir}/icons/breeze-dark/status/22/fcitx-lilypad-off-default-black.svg
%{_datadir}/icons/breeze-dark/status/22/fcitx-lilypad-emoji-default-black.svg
%{_datadir}/icons/breeze-dark/status/22/fcitx-lilypad.svg
%{_datadir}/icons/breeze-dark/status/22/fcitx-lilypad-off.svg
%{_datadir}/icons/breeze-dark/status/22/fcitx-lilypad-emoji.svg

%{_datadir}/icons/breeze-dark/status/24/fcitx-lilypad-default.svg
%{_datadir}/icons/breeze-dark/status/24/fcitx-lilypad-off-default.svg
%{_datadir}/icons/breeze-dark/status/24/fcitx-lilypad-emoji-default.svg
%{_datadir}/icons/breeze-dark/status/24/fcitx-lilypad-default-black.svg
%{_datadir}/icons/breeze-dark/status/24/fcitx-lilypad-off-default-black.svg
%{_datadir}/icons/breeze-dark/status/24/fcitx-lilypad-emoji-default-black.svg
%{_datadir}/icons/breeze-dark/status/24/fcitx-lilypad.svg
%{_datadir}/icons/breeze-dark/status/24/fcitx-lilypad-off.svg
%{_datadir}/icons/breeze-dark/status/24/fcitx-lilypad-emoji.svg

%{_datadir}/metainfo/org.fcitx.Fcitx5.Addon.Lilypad.metainfo.xml

%clean
rm -rf %{buildroot}
rm -rf %{_builddir}/%{name}-%{version}

%post
%systemd_post fcitx5-lilypad-server@.service
if [ -x /usr/bin/udevadm ]; then
    /usr/sbin/modprobe uinput >/dev/null 2>&1 || :
    /usr/bin/udevadm control --reload-rules >/dev/null 2>&1 || :
    /usr/bin/udevadm trigger >/dev/null 2>&1 || :
fi

if [ $1 -eq 1 ]; then
    echo "--- Cấu hình Lilypad ---"
    echo "Hướng dẫn sau cài đặt:"
    echo "1. Kích hoạt Server cho user của bạn:"
    echo "   sudo systemctl enable --now fcitx5-lilypad-server@\$(whoami).service"
    echo ""
    echo "2. Cấu hình Fcitx5:"
    echo "   - Mở 'Fcitx5 Configuration', thêm bộ gõ Lilypad"
    echo ""
    echo "3. Lưu ý cho Wayland (KDE):"
    echo "   - Hãy chọn 'Fcitx 5' trong phần Virtual Keyboard của hệ thống."
    echo "------------------------------------------------"
elif [ $1 -eq 2 ]; then
    echo "--- Cấu hình Lilypad ---"
    echo "Hướng dẫn sau cập nhật:"
    echo "1. Khởi động lại Server cho user của bạn:"
    echo "   sudo systemctl restart fcitx5-lilypad-server@\$(whoami).service"
    echo ""
    echo "2. Cấu hình Fcitx5:"
    echo "   - Mở 'Fcitx5 Configuration', nhấn restart để khởi động lại."
fi


%preun
%systemd_preun fcitx5-lilypad-server@.service

%postun
%systemd_postun_with_restart fcitx5-lilypad-server@.service

%changelog
* Wed Jul 08 2026 Nguyen Hoang Ky <nhktmdzhg@gmail.com> - 3.4.0-1
- Allow macro in off mode
