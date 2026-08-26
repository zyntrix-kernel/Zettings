import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import org.zyntrix.zettings.Style
import org.zyntrix.zettings.Components
import org.zyntrix.zettings.Gallery

ApplicationWindow {
    id: window

    width: 1200
    height: 800
    minimumWidth: ZdlTheme.breakpointOverlay + 40
    visible: true
    title: qsTr("Zettings")
    color: "transparent"

    Rectangle {
        id: auroraBase
        anchors.fill: parent
        color: ZdlTheme.surface

        gradient: Gradient {
            GradientStop { position: 0.0; color: ZdlTheme.darkAppearance ? "#17113a" : "#efe9ff" }
            GradientStop { position: 1.0; color: ZdlTheme.surface }
        }
    }

    Rectangle {
        id: auroraViolet
        anchors.fill: parent
        visible: ZdlTheme.glassEnabled
        opacity: ZdlTheme.darkAppearance ? 0.35 : 0.25

        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0.0; color: Qt.rgba(0.486, 0.227, 0.929, 0.55) }
            GradientStop { position: 0.55; color: "transparent" }
        }
    }

    Rectangle {
        id: auroraTeal
        anchors.fill: parent
        visible: ZdlTheme.glassEnabled
        opacity: ZdlTheme.darkAppearance ? 0.28 : 0.20

        gradient: Gradient {
            orientation: Gradient.Vertical
            GradientStop { position: 0.65; color: "transparent" }
            GradientStop { position: 1.0; color: Qt.rgba(0.055, 0.627, 0.604, 0.50) }
        }
    }

    Frame {
        id: shellFrame
        anchors.fill: parent
        anchors.margins: ZdlTheme.space2
        padding: 0
        background: Squircle {
            radius: ZdlTheme.radiusPanel
            exponent: ZdlTheme.squircleExponentPanel
            fillColor: ZdlTheme.glassEnabled ? Qt.rgba(
                ZdlTheme.surfaceElevated.r, ZdlTheme.surfaceElevated.g,
                ZdlTheme.surfaceElevated.b, ZdlTheme.darkAppearance ? 0.55 : 0.42)
                : ZdlTheme.surfaceElevated
            strokeColor: ZdlTheme.border
            strokeWidth: 1
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: ZdlTheme.space6
            spacing: ZdlTheme.space6

            GalleryPage {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }
    }
}
