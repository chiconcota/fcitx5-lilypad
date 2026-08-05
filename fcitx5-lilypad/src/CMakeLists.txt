set(fcitx_lilypad_sources
  lilypad.cpp
  lilypad-engine.cpp
  lilypad-state.cpp
  lilypad-sequencer.cpp
  lilypad-candidates.cpp
  lilypad-utils.cpp
  lilypad-monitor.cpp
  emoji.cpp
  ack-sensors/niri-sensor.cpp
  ack-sensors/generic-sensor.cpp
  ack-sensors/sensor-factory.cpp
)

add_library(lilypad MODULE ${fcitx_lilypad_sources})
set_target_properties(lilypad PROPERTIES OUTPUT_NAME "lilypad")
target_link_libraries(lilypad
	Fcitx5::Core
	Fcitx5::Config
  Fcitx5::Module::Emoji
	Bamboo::Core
	Pthread::Pthread
	X11::X11
)

target_include_directories(lilypad PRIVATE
    ${PROJECT_BINARY_DIR}
)
target_compile_definitions(lilypad PRIVATE
    FCITX5_LILYPAD_SETTINGS_PATH=\"${CMAKE_INSTALL_FULL_BINDIR}/fcitx5-lilypad-settings\"
)
install(TARGETS lilypad DESTINATION "${CMAKE_INSTALL_LIBDIR}/fcitx5")

fcitx5_translate_desktop_file(lilypad.conf.in lilypad.conf)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/lilypad.conf" DESTINATION "${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod")
configure_file(lilypad-addon.conf.in.in lilypad-addon.conf.in)
fcitx5_translate_desktop_file("${CMAKE_CURRENT_BINARY_DIR}/lilypad-addon.conf.in" lilypad-addon.conf)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/lilypad-addon.conf" RENAME lilypad.conf DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon")