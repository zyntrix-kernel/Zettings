import QtQuick
import org.zyntrix.zettings.Style

Transition {
    NumberAnimation {
        property: "opacity"
        from: 1
        to: 0
        duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick)
    }
    NumberAnimation {
        property: "x"
        from: 0
        to: -16
        duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick)
        enabled: !ZdlTheme.reducedMotion
    }
}
