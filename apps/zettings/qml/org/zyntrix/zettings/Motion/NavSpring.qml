import QtQuick
import org.zyntrix.zettings.Style

SpringAnimation {
    // qmllint disable missing-property
    stiffness: ZdlTheme.springNavigationStiffness
    damping: ZdlTheme.springNavigationDamping
    mass: ZdlTheme.springNavigationMass
    epsilon: 0.01
    velocity: 0
    // qmllint disable missing-property
    enabled: !ZdlTheme.reducedMotion
}
