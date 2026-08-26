import QtQuick
import QtQuick.Layouts
import QtQuick.Controls.Basic
import QtQuick.Shapes
import org.zyntrix.zettings.Style
import org.zyntrix.zettings.Motion

Item {
    id: root

    property string title: ""
    property string description: ""
    default property alias content: contentColumn.children
    property bool expanded: false

    implicitWidth: 320
    implicitHeight: headerCard.implicitHeight
        + (expanded ? ZdlTheme.space2 + contentColumn.implicitHeight : 0)
    clip: true

    Accessible.role: Accessible.ListItem
    Accessible.name: title
    Accessible.checked: expanded

    SettingsCard {
        id: headerCard
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        title: root.title
        description: root.description
        isNavigation: true

        Item {
            Layout.preferredWidth: 20
            Layout.preferredHeight: 20
            Layout.alignment: Qt.AlignVCenter

            Shape {
                anchors.fill: parent
                antialiasing: true
                rotation: root.expanded ? 90 : 0

                Behavior on rotation {
                    NumberAnimation {
                        duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick)
                        easing.type: Easing.OutCubic
                    }
                }

                ShapePath {
                    strokeColor: ZdlTheme.textMuted
                    strokeWidth: 2
                    fillColor: "transparent"
                    joinStyle: ShapePath.Round
                    capStyle: ShapePath.Round

                    PathMove { x: 6; y: 3 }
                    PathLine { x: 12; y: 10 }
                    PathLine { x: 6; y: 17 }
                }
            }
        }

        onActivated: root.expanded = !root.expanded
    }

    ColumnLayout {
        id: contentColumn
        anchors.top: headerCard.bottom
        anchors.topMargin: ZdlTheme.space2
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: ZdlTheme.space2
        visible: root.expanded

        opacity: root.expanded ? 1 : 0
        y: root.expanded ? 0 : -8

        Behavior on opacity {
            enabled: !ZdlTheme.reducedMotion
            NumberAnimation { duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick) }
        }

        Behavior on y {
            enabled: !ZdlTheme.reducedMotion
            ControlSpring {}
        }
    }
}
