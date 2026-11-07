find_package(PkgConfig REQUIRED)

pkg_check_modules(LIBINPUT REQUIRED IMPORTED_TARGET libinput)
pkg_check_modules(LIBUDEV  REQUIRED IMPORTED_TARGET libudev)

add_executable(fcitx5-lilypad-server lilypad-server.cpp lilypad-logger.cpp)

target_link_libraries(fcitx5-lilypad-server
    PkgConfig::LIBINPUT
    PkgConfig::LIBUDEV
)

target_include_directories(fcitx5-lilypad-server PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../src
)

install(TARGETS fcitx5-lilypad-server DESTINATION ${CMAKE_INSTALL_BINDIR})
