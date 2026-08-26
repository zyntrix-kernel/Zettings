import QtQuick
import org.zyntrix.zettings.Style
import QtQuick.Layouts
import QtQuick.Controls.Basic
import QtQuick.Shapes

Rectangle {
    id: root

    enum Severity {
        Info,
        Warning,
        Danger
    }

    property int severity: InfoBar.Severity.Info
    property string message: ""
    property alias actionText: actionButton.text

    signal actionTriggered
    signal dismissed

    implicitWidth: 480
    implicitHeight: Math.max(ZdlTheme.rowMinHeight, contentLayout.implicitHeight + ZdlTheme.space3 * 2)
    radius: ZdlTheme.radiusCard

    color: ZdlTheme.surfaceElevated
    border.width: 1
    border.color: root.severity === InfoBar.Severity.Warning ? ZdlTheme.warning : (root.severity === InfoBar.Severity.Danger ? ZdlTheme.danger : ZdlTheme.border)

    Accessible.role: Accessible.Alert
    Accessible.name: message

    readonly property color severityColor: root.severity === InfoBar.Severity.Warning ? ZdlTheme.warning : (root.severity === InfoBar.Severity.Danger ? ZdlTheme.danger : ZdlTheme.accent)

    RowLayout {
        id: contentLayout
        anchors.fill: parent
        anchors.leftMargin: ZdlTheme.space4
        anchors.rightMargin: ZdlTheme.space3
        anchors.topMargin: ZdlTheme.space2
        anchors.bottomMargin: ZdlTheme.space2
        spacing: ZdlTheme.space3

        Rectangle {
            Layout.preferredWidth: 8
            Layout.preferredHeight: 8
            Layout.alignment: Qt.AlignVCenter
            radius: 4
            color: root.severityColor
        }

        Label {
            text: root.message
            font.family: ZdlTheme.fontFamily
            font.pixelSize: ZdlTheme.textBodySize
            color: ZdlTheme.text
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }

        Button {
            id: actionButton
            visible: text !== ""
            // qmllint disable missing-property
            flat: true
            font.family: ZdlTheme.fontFamily
            font.pixelSize: ZdlTheme.textBodySize
            palette.buttonText: root.severityColor
            activeFocusOnTab: true
            Layout.alignment: Qt.AlignVCenter
            onClicked: root.actionTriggered()
        }

        AbstractButton {
            id: dismissButton
            // qmllint disable missing-property
            flat: true
            activeFocusOnTab: true
            Accessible.role: Accessible.Button
            Accessible.name: qsTr("Dismiss")
            padding: 4
            Layout.alignment: Qt.AlignVCenter

            contentItem: Item {
                implicitWidth: 14
                implicitHeight: 14

                Shape {
                    anchors.fill: parent
                    antialiasing: true

                    ShapePath {
                        strokeColor: hoverHandler.hovered ? ZdlTheme.text : ZdlTheme.textMuted
                        strokeWidth: 2
                        fillColor: "transparent"
                        // qmllint disable missing-property
                        capStyle: ShapePath.Round

                        PathMove {
                            x: 2
                            y: 2
                        }
                        PathLine {
                            x: 12
                            y: 12
                        }
                        PathMove {
                            x: 12
                            y: 2
                        }
                        PathLine {
                            x: 2
                            y: 12
                        }
                    }
                }

                HoverHandler {
                    id: hoverHandler
                }
            }

            onClicked: root.dismissed()
        }
    }
}
