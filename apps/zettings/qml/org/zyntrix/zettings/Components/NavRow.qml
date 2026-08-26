import QtQuick
import org.zyntrix.zettings.Style
import org.zyntrix.zettings.Motion
import QtQuick.Layouts
import QtQuick.Controls.Basic

Item {
    id: root

    property string label: ""
    property url iconSource: ""
    property string badge: ""
    property bool selected: false

    signal activated

    implicitWidth: 240
    implicitHeight: ZdlTheme.rowMinHeight
    activeFocusOnTab: true
    Accessible.role: Accessible.ListItem
    Accessible.name: label + (badge !== "" ? ", " + badge : "")
    Accessible.selected: selected

    scale: tapHandler.pressed && !ZdlTheme.reducedMotion ? 0.98 : 1.0

    Behavior on scale {
        enabled: !ZdlTheme.reducedMotion
        ControlSpring {}
    }

    Squircle {
        anchors.fill: parent
        radius: ZdlTheme.radiusControl
        exponent: ZdlTheme.squircleExponentControl
        fillColor: root.selected ? ZdlTheme.accentSoft : (hoverHandler.hovered ? ZdlTheme.surfaceMuted : "transparent")
        strokeColor: root.activeFocus ? ZdlTheme.focusRingColor : "transparent"
        strokeWidth: 3

        Behavior on fillColor {
            ColorAnimation {
                duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick)
            }
        }
    }

    HoverHandler {
        id: hoverHandler
        cursorShape: Qt.PointingHandCursor
    }

    TapHandler {
        id: tapHandler
        onTapped: root.activated()
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: ZdlTheme.space3
        anchors.rightMargin: ZdlTheme.space3
        spacing: ZdlTheme.space3

        Image {
            visible: root.iconSource.toString() !== ""
            source: root.iconSource
            sourceSize: Qt.size(20, 20)
            Layout.preferredWidth: 20
            Layout.preferredHeight: 20
            fillMode: Image.PreserveAspectFit
            asynchronous: true
        }

        Label {
            text: root.label
            font.family: ZdlTheme.fontFamily
            font.pixelSize: ZdlTheme.textBodySize
            font.weight: root.selected ? Font.DemiBold : Font.Normal
            color: root.selected ? ZdlTheme.text : ZdlTheme.textMuted
            elide: Text.ElideRight
            Layout.fillWidth: true
        }

        Rectangle {
            visible: root.badge !== ""
            radius: height / 2
            color: ZdlTheme.accent
            implicitWidth: Math.max(badgeLabel.implicitWidth + ZdlTheme.space2 * 2, 20)
            implicitHeight: 20
            Layout.alignment: Qt.AlignVCenter

            Label {
                id: badgeLabel
                anchors.centerIn: parent
                text: root.badge
                font.family: ZdlTheme.fontFamily
                font.pixelSize: ZdlTheme.textCaptionSize
                font.weight: Font.DemiBold
                color: ZdlTheme.accentText
            }
        }
    }

    Keys.onSpacePressed: event => {
        root.activated();
        event.accepted = true;
    }
    Keys.onReturnPressed: event => {
        root.activated();
        event.accepted = true;
    }
}
