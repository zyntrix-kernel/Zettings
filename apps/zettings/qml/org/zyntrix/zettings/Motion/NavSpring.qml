import QtQuick
import org.zyntrix.zettings.Style

SpringAnimation {
    stiffness: ZdlTheme.springNavigationStiffness
    damping: ZdlTheme.springNavigationDamping
    mass: ZdlTheme.springNavigationMass
    epsilon: 0.01
    velocity: 0
    enabled: !ZdlTheme.reducedMotion
}
