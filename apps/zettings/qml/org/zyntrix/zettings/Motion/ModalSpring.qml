import QtQuick
import org.zyntrix.zettings.Style

SpringAnimation {
    // qmllint disable missing-property
    stiffness: ZdlTheme.springModalStiffness
    damping: ZdlTheme.springModalDamping
    mass: ZdlTheme.springModalMass
    epsilon: 0.01
    velocity: 0
    // qmllint disable missing-property
    enabled: !ZdlTheme.reducedMotion
}
