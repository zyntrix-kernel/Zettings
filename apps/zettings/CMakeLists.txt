cmake_minimum_required(VERSION 3.21)

project(zettings
    VERSION 0.1.0
    DESCRIPTION "Zettings — system settings for Zyntrix OS"
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.4 REQUIRED COMPONENTS Quick)
qt_standard_project_setup()

qt_add_executable(zettings
    src/main.cpp
)

qt_add_qml_module(zettings
    URI org.zyntrix.zettings
    VERSION 1.0
    QML_FILES
        qml/org/zyntrix/zettings/Main.qml
    DEPENDENCIES
        QtQuick
        org.zyntrix.zettings.Style
        org.zyntrix.zettings.Components
        org.zyntrix.zettings.Gallery
)

set_source_files_properties(qml/org/zyntrix/zettings/Style/ZdlTheme.qml
    PROPERTIES QT_QML_SINGLETON_TYPE TRUE
)

qt_add_qml_module(zettings
    URI org.zyntrix.zettings.Style
    VERSION 1.0
    QML_FILES
        qml/org/zyntrix/zettings/Style/ZdlTheme.qml
    DEPENDENCIES
        QtQuick
)

qt_add_qml_module(zettings
    URI org.zyntrix.zettings.Motion
    VERSION 1.0
    QML_FILES
        qml/org/zyntrix/zettings/Motion/MotionEase.qml
        qml/org/zyntrix/zettings/Motion/ControlSpring.qml
        qml/org/zyntrix/zettings/Motion/NavSpring.qml
        qml/org/zyntrix/zettings/Motion/ModalSpring.qml
        qml/org/zyntrix/zettings/Motion/PageEnter.qml
        qml/org/zyntrix/zettings/Motion/PageExit.qml
    DEPENDENCIES
        QtQuick
        org.zyntrix.zettings.Style
)

qt_add_qml_module(zettings
    URI org.zyntrix.zettings.Components
    VERSION 1.0
    QML_FILES
        qml/org/zyntrix/zettings/Components/Squircle.qml
        qml/org/zyntrix/zettings/Components/SettingsCard.qml
        qml/org/zyntrix/zettings/Components/SettingsExpander.qml
        qml/org/zyntrix/zettings/Components/NavRow.qml
        qml/org/zyntrix/zettings/Components/ToggleSwitch.qml
        qml/org/zyntrix/zettings/Components/InfoBar.qml
    DEPENDENCIES
        QtQuick
        QtQuick.Controls.Basic
        QtQuick.Shapes
        QtQuick.Layouts
        org.zyntrix.zettings.Style
        org.zyntrix.zettings.Motion
)

qt_add_qml_module(zettings
    URI org.zyntrix.zettings.Gallery
    VERSION 1.0
    QML_FILES
        qml/org/zyntrix/zettings/Gallery/GalleryPage.qml
    DEPENDENCIES
        QtQuick
        QtQuick.Controls.Basic
        QtQuick.Layouts
        org.zyntrix.zettings.Style
        org.zyntrix.zettings.Components
)

target_link_libraries(zettings
    PRIVATE
        Qt6::Quick
)
