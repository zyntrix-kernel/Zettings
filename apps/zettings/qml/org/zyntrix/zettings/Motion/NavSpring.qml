import QtQuick
import org.zyntrix.zettings.Style

SpringAnimation {
    spring: ZdlTheme.springNavigationStrength / 10
    damping: ZdlTheme.springNavigationDamping / 100
    epsilon: 0.01
    velocity: 0
}
