import QtQuick
import QtQuick.Controls.Basic
import org.zyntrix.zettings.Style

Switch {
    id: root

    implicitWidth: 44
    implicitHeight: 44
    activeFocusOnTab: true

    padding: 0
    spacing: 0

    readonly property int trackWidth: 40
    readonly property int trackHeight: 24

    indicator: Rectangle {
        implicitWidth: root.trackWidth
        implicitHeight: root.trackHeight
        x: (root.width - width) / 2
        y: (root.height - height) / 2
        radius: height / 2
        color: root.checked ? ZdlTheme.accent : "transparent"
        border.width: root.activeFocus ? 3 : 1
        border.color: root.activeFocus ? ZdlTheme.focusRingColor : (root.enabled ? ZdlTheme.borderStrong : ZdlTheme.border)

        Behavior on color {
            enabled: !ZdlTheme.reducedMotion
            ColorAnimation {
                duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick)
            }
        }

        Rectangle {
            readonly property int knobTravel: parent.width - width - 4
            x: root.checked ? knobTravel + 2 : 2
            y: (parent.height - height) / 2
            width: 20
            height: 20
            radius: width / 2
            color: root.checked ? ZdlTheme.accentText : (root.enabled ? ZdlTheme.surfaceElevated : ZdlTheme.surfaceMuted)
            border.width: root.checked ? 0 : 1
            border.color: ZdlTheme.borderStrong

            Behavior on x {
                NumberAnimation {
                    duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick)
                    easing.type: Easing.OutCubic
                }
            }
        }
    }

    // qmllint disable missing-property
    Accessible.role: Accessible.Switch
    Accessible.name: text !== "" ? text : qsTr("Toggle")
    Accessible.checked: checked
}
