option(INSTALL_OPENRC "Install OpenRC script" OFF)
option(INSTALL_RUNIT "Install Runit script" OFF)

install(FILES fcitx5-lilypad-server@.service
    DESTINATION ${CMAKE_INSTALL_PREFIX}/lib/systemd/system)

install(FILES 99-lilypad.rules
    DESTINATION ${CMAKE_INSTALL_PREFIX}/lib/udev/rules.d)

install(FILES module-fcitx5-lilypad.conf
    DESTINATION ${CMAKE_INSTALL_PREFIX}/lib/modules-load.d
    RENAME fcitx5-lilypad.conf)

install(FILES user-lilypad.conf
    DESTINATION ${CMAKE_INSTALL_PREFIX}/lib/sysusers.d
    RENAME lilypad.conf)

if(INSTALL_OPENRC)
    install(PROGRAMS fcitx5-lilypad.openrc
        DESTINATION /etc/init.d
        RENAME fcitx5-lilypad)
endif(INSTALL_OPENRC)

if(INSTALL_RUNIT)
    set(RUNIT_SV_DIR "/etc/sv" CACHE PATH
        "Path to runit service definitions directory (e.g. /etc/sv on Void, /etc/runit/sv on Artix)")
    install(PROGRAMS fcitx5-lilypad.runit
        DESTINATION "${RUNIT_SV_DIR}/fcitx5-lilypad"
        RENAME run)
endif(INSTALL_RUNIT)
