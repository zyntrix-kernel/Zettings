import QtQuick
import org.zyntrix.zettings.Style
import org.zyntrix.zettings.Motion

Transition {
    NumberAnimation {
        property: "opacity"
        from: 0
        to: 1
        duration: ZdlTheme.motionDuration(ZdlTheme.motionQuick)
    }
    NumberAnimation {
        property: "x"
        from: 24
        to: 0
        easing.type: Easing.BezierSpline
        easing.bezierCurve: [0.2, 0, 0, 1, 1, 1]
        duration: ZdlTheme.reducedMotion ? 0 : ZdlTheme.motionNormal
    }
}
