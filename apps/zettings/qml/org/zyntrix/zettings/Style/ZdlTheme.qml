pragma Singleton
import QtQuick

Item {
    id: root

    enum ThemeMode { Light, Dark, Oled, HighContrast }

    property int themeMode: ZdlTheme.Dark

    readonly property bool isHighContrast: themeMode === ZdlTheme.ThemeMode.HighContrast
    readonly property bool glassEnabled: !isHighContrast && themeMode !== ZdlTheme.ThemeMode.Oled
    readonly property bool darkAppearance: themeMode !== ZdlTheme.ThemeMode.Light

    readonly property string base50: "#fafafa"
    readonly property string base100: "#f4f4f5"
    readonly property string base200: "#e4e4e7"
    readonly property string base300: "#d4d4d8"
    readonly property string base400: "#a1a1aa"
    readonly property string base500: "#71717a"
    readonly property string base600: "#52525b"
    readonly property string base700: "#3f3f46"
    readonly property string base800: "#27272a"
    readonly property string base900: "#18181b"
    readonly property string base950: "#09090b"

    readonly property color accentLight: "#7c3aed"
    readonly property color accentDark: "#a78bfa"
    readonly property color accentStrongLight: "#6d28d9"
    readonly property color accentStrongDark: "#c4b5fd"
    readonly property color onAccentLight: "#ffffff"
    readonly property color onAccentDark: "#241145"

    readonly property color warningLight: "#b45309"
    readonly property color warningDark: "#fbbf24"
    readonly property color dangerLight: "#b91c1c"
    readonly property color dangerDark: "#f87171"

    readonly property color accent: darkAppearance ? accentDark : accentLight
    readonly property color accentStrong: darkAppearance ? accentStrongDark : accentStrongLight
    readonly property color onAccent: darkAppearance ? onAccentDark : onAccentLight
    readonly property color accentSoft: Qt.rgba(accent.r, accent.g, accent.b, isHighContrast ? 0.0 : (darkAppearance ? 0.16 : 0.12))

    readonly property color surface: {
        if (themeMode === ZdlTheme.ThemeMode.Light) return base50
        if (themeMode === ZdlTheme.ThemeMode.Dark) return base900
        if (themeMode === ZdlTheme.ThemeMode.Oled) return "#000000"
        return "#000000"
    }
    readonly property color surfaceElevated: {
        if (themeMode === ZdlTheme.ThemeMode.Light) return "#ffffff"
        if (themeMode === ZdlTheme.ThemeMode.Dark) return base800
        if (themeMode === ZdlTheme.ThemeMode.Oled) return "#101010"
        return "#ffffff"
    }
    readonly property color surfaceMuted: {
        if (themeMode === ZdlTheme.ThemeMode.Light) return base100
        if (themeMode === ZdlTheme.ThemeMode.Dark) return Qt.rgba(base800.r, base800.g, base800.b, 0.6)
        if (themeMode === ZdlTheme.ThemeMode.Oled) return "#0a0a0a"
        return "#000000"
    }
    readonly property color surfaceSunken: {
        if (themeMode === ZdlTheme.ThemeMode.Light) return base200
        if (themeMode === ZdlTheme.ThemeMode.Dark) return base950
        return "#000000"
    }
    readonly property color text: {
        if (isHighContrast) return "#ffffff"
        if (darkAppearance) return base50
        return base900
    }
    readonly property color textMuted: darkAppearance ? base400 : base600
    readonly property color textSubtle: base500
    readonly property color border: {
        if (isHighContrast) return "#ffffff"
        if (themeMode === ZdlTheme.ThemeMode.Light) return base300
        if (themeMode === ZdlTheme.ThemeMode.Dark) return base700
        return base800
    }
    readonly property color borderStrong: {
        if (isHighContrast) return "#ffffff"
        if (themeMode === ZdlTheme.ThemeMode.Oled) return base600
        return base500
    }
    readonly property color focusRingColor: accent
    readonly property color shadowTint: {
        if (themeMode === ZdlTheme.ThemeMode.Light) return Qt.rgba(0, 0, 0, 0.12)
        if (themeMode === ZdlTheme.ThemeMode.Dark) return Qt.rgba(0, 0, 0, 0.40)
        if (themeMode === ZdlTheme.ThemeMode.Oled) return Qt.rgba(0, 0, 0, 0.60)
        return Qt.rgba(0, 0, 0, 0.60)
    }

    readonly property string warning: darkAppearance ? warningDark : warningLight
    readonly property string danger: darkAppearance ? dangerDark : dangerLight

    readonly property string fontFamily: "Inter"
    readonly property string fontFamilyFallback: "Cantarell"
    readonly property string fontFamilyMono: "JetBrains Mono"

    readonly property int textDisplaySize: 28
    readonly property int textDisplayLineHeight: 36
    readonly property int textTitleLgSize: 20
    readonly property int textTitleLgLineHeight: 28
    readonly property int textTitleSize: 16
    readonly property int textTitleLineHeight: 24
    readonly property int textBodySize: 14
    readonly property int textBodyLineHeight: 21
    readonly property int textCaptionSize: 12
    readonly property int textCaptionLineHeight: 18

    readonly property int space1: 4
    readonly property int space2: 8
    readonly property int space3: 12
    readonly property int space4: 16
    readonly property int space5: 20
    readonly property int space6: 24
    readonly property int space8: 32
    readonly property int space10: 40
    readonly property int space12: 48
    readonly property int space16: 64

    readonly property int radiusControl: 10
    readonly property int radiusCard: 14
    readonly property int radiusPanel: 20
    readonly property int radiusOverlay: 28

    readonly property real squircleExponentControl: 4.0
    readonly property real squircleExponentPanel: 6.0

    readonly property int elevationRestBlur: 2
    readonly property int elevationHoverBlur: 8
    readonly property int elevationDialogBlur: 24
    readonly property int elevationOverlayBlur: 48

    readonly property int densityComfortableRowHeight: 56
    readonly property int densityCompactRowHeight: 44
    property bool compactDensity: false
    readonly property int rowMinHeight: compactDensity
        ? densityCompactRowHeight
        : densityComfortableRowHeight

    readonly property int contentMaxWidth: 1000
    readonly property int breakpointExpanded: 1100
    readonly property int breakpointRail: 800
    readonly property int breakpointOverlay: 560

    readonly property int motionInstant: 80
    readonly property int motionQuick: 140
    readonly property int motionNormal: 220
    readonly property int motionDeliberate: 320

    readonly property int springNavigationStiffness: 220
    readonly property int springNavigationDamping: 28
    readonly property real springNavigationMass: 1.0

    readonly property int springControlsStiffness: 320
    readonly property int springControlsDamping: 22
    readonly property real springControlsMass: 0.6

    readonly property int springModalStiffness: 180
    readonly property int springModalDamping: 24
    readonly property real springModalMass: 1.2

    property bool debugFrames: false

    readonly property real frameBudgetMs: 8.3
    property real frameAverageMs: 0
    property real frameWorstMs: 0

    property bool _primedFrames: false
    property int _windowSamples: 0

    FrameAnimation {
        running: true

        onTriggered: {
            if (!ZdlTheme._primedFrames) {
                ZdlTheme._primedFrames = true
                return
            }
            const dtMs = (frameTime - previousFrameTime) * 1000
            if (dtMs <= 0 || dtMs > 500)
                return
            ZdlTheme.frameAverageMs = ZdlTheme.frameAverageMs * 0.9 + dtMs * 0.1
            if (dtMs > ZdlTheme.frameWorstMs || ZdlTheme._windowSamples === 0)
                ZdlTheme.frameWorstMs = dtMs
            ZdlTheme._windowSamples++
            if (ZdlTheme.debugFrames && ZdlTheme._windowSamples >= 300) {
                console.info("frame stats:",
                             "avg", ZdlTheme.frameAverageMs.toFixed(2), "ms",
                             "worst(300f)", ZdlTheme.frameWorstMs.toFixed(2), "ms",
                             "budget", ZdlTheme.frameBudgetMs, "ms")
                ZdlTheme._windowSamples = 0
                ZdlTheme.frameWorstMs = 0
            }
        }
    }

    property bool reducedMotion: false

    function motionDuration(preferred) {
        if (!reducedMotion)
            return preferred
        return Math.min(preferred, 120)
    }
}
