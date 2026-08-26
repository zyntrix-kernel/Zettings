import QtQuick
import org.zyntrix.zettings.Style

SpringAnimation {
    // qmllint disable missing-property
    stiffness: ZdlTheme.springControlsStiffness
    damping: ZdlTheme.springControlsDamping
    mass: ZdlTheme.springControlsMass
    epsilon: 0.01
    velocity: 0
    // qmllint disable missing-property
    enabled: !ZdlTheme.reducedMotion
}
