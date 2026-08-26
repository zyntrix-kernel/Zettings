import QtQuick
import org.zyntrix.zettings.Style

SpringAnimation {
    spring: ZdlTheme.springModalStrength / 10
    damping: ZdlTheme.springModalDamping / 100
    epsilon: 0.01
    velocity: 0
}
