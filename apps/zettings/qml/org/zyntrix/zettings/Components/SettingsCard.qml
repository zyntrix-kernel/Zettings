import QtQuick
import QtQuick.Layouts
import QtQuick.Controls.Basic
import QtQuick.Shapes
import org.zyntrix.zettings.Style

Item {
    id: root

    property string title: ""
    property string description: ""
    property url iconSource: ""
    property alias control: controlSlot.children
    property bool isNavigation: false
    property bool selected: false

    signal activated()

    implicitWidth: 320
    implicitHeight: Math.max(ZdlTheme.rowMinHeight, contentLayout.implicitHeight + ZdlTheme.space4 * 2)
    activeFocusOnTab: true
    Accessible.role: Accessible.ListItem
    Accessible.name: title + (description !== "" ? ". " + description : "")
    Accessible.selected: selected

    Squircle {
        id: background
        anchors.fill: parent
        radius: ZdlTheme.radiusCard
        exponent: ZdlTheme.squircleExponentControl
        fillColor: root.selected ? ZdlTheme.accentSoft
            : (hoverHandler.hovered && !root.isNavigation ? ZdlTheme.surfaceElevated : ZdlTheme.surfaceMuted)
        strokeColor: root.activeFocus ? ZdlTheme.focusRingColor : ZdlTheme.border
        strokeWidth: root.activeFocus ? 3 : 1

        Behavior on fillColor {
            ColorAnimation { duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick) }
        }
    }

    HoverHandler {
        id: hoverHandler
        cursorShape: root.isNavigation ? Qt.PointingHandCursor : Qt.ArrowCursor
    }

    TapHandler {
        enabled: root.isNavigation
        onTapped: root.activated()
    }

    RowLayout {
        id: contentLayout
        anchors.fill: parent
        anchors.leftMargin: ZdlTheme.space4
        anchors.rightMargin: ZdlTheme.space4
        anchors.topMargin: ZdlTheme.space3
        anchors.bottomMargin: ZdlTheme.space3
        spacing: ZdlTheme.space4

        Image {
            visible: root.iconSource.toString() !== ""
            source: root.iconSource
            sourceSize: Qt.size(24, 24)
            Layout.preferredWidth: 24
            Layout.preferredHeight: 24
            Layout.alignment: Qt.AlignVCenter
            fillMode: Image.PreserveAspectFit
            asynchronous: true
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            Label {
                text: root.title
                font.family: ZdlTheme.fontFamily
                font.pixelSize: ZdlTheme.textTitleSize
                font.weight: Font.DemiBold
                color: root.enabled ? ZdlTheme.text : ZdlTheme.textSubtle
                elide: Text.ElideRight
                Layout.fillWidth: true
            }

            Label {
                visible: root.description !== ""
                text: root.description
                font.family: ZdlTheme.fontFamily
                font.pixelSize: ZdlTheme.textBodySize
                color: ZdlTheme.textMuted
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        ColumnLayout {
            id: controlSlot
            Layout.alignment: Qt.AlignVCenter
            spacing: 0
        }

        Shape {
            visible: root.isNavigation
            Layout.preferredWidth: 16
            Layout.preferredHeight: 16
            Layout.alignment: Qt.AlignVCenter
            antialiasing: true

            ShapePath {
                strokeColor: ZdlTheme.textMuted
                strokeWidth: 2
                fillColor: "transparent"
                joinStyle: ShapePath.Round
                capStyle: ShapePath.Round

                PathMove { x: 5; y: 2 }
                PathLine { x: 12; y: 8 }
                PathLine { x: 5; y: 14 }
            }
        }
    }

    Keys.onSpacePressed: if (root.isNavigation) { root.activated(); event.accepted = true }
    Keys.onReturnPressed: if (root.isNavigation) { root.activated(); event.accepted = true }
}
