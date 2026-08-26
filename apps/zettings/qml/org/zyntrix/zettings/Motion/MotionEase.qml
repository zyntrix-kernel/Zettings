import QtQuick
import org.zyntrix.zettings.Style

NumberAnimation {
    duration: ZdlTheme.motionDuration(ZdlTheme.motionNormal)
    easing.type: Easing.BezierSpline
    easing.bezierCurve: [0.2, 0, 0, 1, 1, 1]
    alwaysRunToEnd: false
}
