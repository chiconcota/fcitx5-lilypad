option(INSTALL_OPENRC "Install OpenRC script" OFF)
option(INSTALL_RUNIT "Install Runit script" OFF)

install(FILES fcitx5-lotus-server@.service
    DESTINATION ${CMAKE_INSTALL_PREFIX}/lib/systemd/system)

install(FILES 99-lotus.rules
    DESTINATION ${CMAKE_INSTALL_PREFIX}/lib/udev/rules.d)

install(FILES module-fcitx5-lotus.conf
    DESTINATION ${CMAKE_INSTALL_PREFIX}/lib/modules-load.d
    RENAME fcitx5-lotus.conf)

install(FILES user-lotus.conf
    DESTINATION ${CMAKE_INSTALL_PREFIX}/lib/sysusers.d
    RENAME lotus.conf)

if(INSTALL_OPENRC)
    install(PROGRAMS fcitx5-lotus.openrc
        DESTINATION /etc/init.d
        RENAME fcitx5-lotus)
endif(INSTALL_OPENRC)

if(INSTALL_RUNIT)
    set(RUNIT_SV_DIR "/etc/sv" CACHE PATH
        "Path to runit service definitions directory (e.g. /etc/sv on Void, /etc/runit/sv on Artix)")
    install(PROGRAMS fcitx5-lotus.runit
        DESTINATION "${RUNIT_SV_DIR}/fcitx5-lotus"
        RENAME run)
endif(INSTALL_RUNIT)
