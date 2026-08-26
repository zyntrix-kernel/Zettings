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
    readonly property int trackX: (width - trackWidth) / 2
    readonly property int trackY: (height - trackHeight) / 2

    indicator: Rectangle {
        implicitWidth: root.trackWidth
        implicitHeight: root.trackHeight
        x: root.trackX
        y: root.trackY
        radius: height / 2
        color: root.checked ? ZdlTheme.accent : "transparent"
        border.width: root.activeFocus ? 3 : 1
        border.color: root.activeFocus ? ZdlTheme.focusRingColor
            : (root.enabled ? ZdlTheme.borderStrong : ZdlTheme.border)

        Behavior on color {
            enabled: !ZdlTheme.reducedMotion
            ColorAnimation { duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick) }
        }
    }

    handle: Rectangle {
        implicitWidth: 20
        implicitHeight: 20
        x: root.checked ? root.trackX + root.trackWidth - width - 2 : root.trackX + 2
        y: root.trackY + (root.trackHeight - height) / 2
        radius: width / 2
        color: root.checked ? ZdlTheme.onAccent
            : (root.enabled ? ZdlTheme.surfaceElevated : ZdlTheme.surfaceMuted)
        border.width: root.checked ? 0 : 1
        border.color: ZdlTheme.borderStrong

        Behavior on x {
            NumberAnimation {
                duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick)
                easing.type: Easing.OutCubic
            }
        }
    }

    Accessible.role: Accessible.Switch
    Accessible.name: text !== "" ? text : qsTr("Toggle")
    Accessible.checked: checked
}
