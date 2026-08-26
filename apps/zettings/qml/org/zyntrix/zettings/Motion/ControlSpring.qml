import QtQuick
import org.zyntrix.zettings.Style

SpringAnimation {
    spring: ZdlTheme.springControlsStrength / 10
    damping: ZdlTheme.springControlsDamping / 100
    epsilon: 0.01
    velocity: 0
}
