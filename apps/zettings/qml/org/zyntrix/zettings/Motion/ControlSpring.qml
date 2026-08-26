import QtQuick
import org.zyntrix.zettings.Style

SpringAnimation {
    stiffness: ZdlTheme.springControlsStiffness
    damping: ZdlTheme.springControlsDamping
    mass: ZdlTheme.springControlsMass
    epsilon: 0.01
    velocity: 0
    enabled: !ZdlTheme.reducedMotion
}
