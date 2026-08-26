cmake_minimum_required(VERSION 3.21)

project(zettings
    VERSION 0.1.0
    DESCRIPTION "Zettings â€” system settings for Zyntrix OS"
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.4 REQUIRED COMPONENTS Quick)
qt_standard_project_setup()


set_source_files_properties(qml/org/zyntrix/zettings/Style/ZdlTheme.qml
    PROPERTIES
        QT_QML_SINGLETON_TYPE TRUE
        QT_RESOURCE_ALIAS ZdlTheme.qml
)

qt_add_library(zettings_style STATIC)
qt_add_qml_module(zettings_style
    URI org.zyntrix.zettings.Style
    OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/org/zyntrix/zettings/Style
    VERSION 1.0
    QML_FILES
        qml/org/zyntrix/zettings/Style/ZdlTheme.qml
)

qt_add_library(zettings_motion STATIC)
qt_add_qml_module(zettings_motion
    URI org.zyntrix.zettings.Motion
    OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/org/zyntrix/zettings/Motion
    VERSION 1.0
    QML_FILES
        qml/org/zyntrix/zettings/Motion/MotionEase.qml
        qml/org/zyntrix/zettings/Motion/ControlSpring.qml
        qml/org/zyntrix/zettings/Motion/NavSpring.qml
        qml/org/zyntrix/zettings/Motion/ModalSpring.qml
        qml/org/zyntrix/zettings/Motion/PageEnter.qml
        qml/org/zyntrix/zettings/Motion/PageExit.qml
)
target_link_libraries(zettings_motion PUBLIC
    zettings_styleplugin
)

qt_add_library(zettings_components STATIC)
qt_add_qml_module(zettings_components
    URI org.zyntrix.zettings.Components
    OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/org/zyntrix/zettings/Components
    VERSION 1.0
    QML_FILES
        qml/org/zyntrix/zettings/Components/Squircle.qml
        qml/org/zyntrix/zettings/Components/SettingsCard.qml
        qml/org/zyntrix/zettings/Components/SettingsExpander.qml
        qml/org/zyntrix/zettings/Components/NavRow.qml
        qml/org/zyntrix/zettings/Components/ToggleSwitch.qml
        qml/org/zyntrix/zettings/Components/InfoBar.qml
)
target_link_libraries(zettings_components PUBLIC
    zettings_styleplugin
    zettings_motionplugin
)

qt_add_executable(zettings
    src/main.cpp
)

qt_add_qml_module(zettings
    URI org.zyntrix.zettings
    VERSION 1.0
    DEPENDENCIES QtQuick org.zyntrix.zettings.Style org.zyntrix.zettings.Components org.zyntrix.zettings.Gallery
    OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/org/zyntrix/zettings
    QML_FILES
        qml/org/zyntrix/zettings/Main.qml
)

target_link_libraries(zettings PRIVATE
    Qt6::Quick
    zettings_styleplugin
    zettings_motionplugin
    zettings_componentsplugin
    zettings_galleryplugin
)

qt_add_library(zettings_gallery STATIC)
qt_add_qml_module(zettings_gallery
    URI org.zyntrix.zettings.Gallery
    OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/org/zyntrix/zettings/Gallery
    VERSION 1.0
    QML_FILES
        qml/org/zyntrix/zettings/Gallery/GalleryPage.qml
)
target_link_libraries(zettings_gallery PUBLIC
    zettings_styleplugin
    zettings_motionplugin
    zettings_componentsplugin
)
