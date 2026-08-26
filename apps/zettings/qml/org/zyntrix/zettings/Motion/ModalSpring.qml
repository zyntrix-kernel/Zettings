import QtQuick
import org.zyntrix.zettings.Style

SpringAnimation {
    stiffness: ZdlTheme.springModalStiffness
    damping: ZdlTheme.springModalDamping
    mass: ZdlTheme.springModalMass
    epsilon: 0.01
    velocity: 0
    enabled: !ZdlTheme.reducedMotion
}
