import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import org.zyntrix.zettings.Style
import org.zyntrix.zettings.Components

Page {
    id: root

    background: null
    padding: 0

    header: ColumnLayout {
        spacing: ZdlTheme.space2

        Label {
            text: qsTr("Zyntrix Design Language")
            font.family: ZdlTheme.fontFamily
            font.pixelSize: ZdlTheme.textDisplaySize
            font.weight: Font.DemiBold
            color: ZdlTheme.text
        }

        RowLayout {
            spacing: ZdlTheme.space2

            Repeater {
                model: [
                    { label: qsTr("Light"), mode: ZdlTheme.ThemeMode.Light },
                    { label: qsTr("Dark"), mode: ZdlTheme.ThemeMode.Dark },
                    { label: qsTr("OLED"), mode: ZdlTheme.ThemeMode.Oled },
                    { label: qsTr("High contrast"), mode: ZdlTheme.ThemeMode.HighContrast }
                ]

                Button {
                    required property var modelData
                    text: modelData.label
                    checkable: true
                    checked: ZdlTheme.themeMode === modelData.mode
                    font.family: ZdlTheme.fontFamily
                    font.pixelSize: ZdlTheme.textCaptionSize
                    activeFocusOnTab: true
                    onClicked: ZdlTheme.themeMode = modelData.mode
                }
            }

            Item { Layout.fillWidth: true }

            ToggleSwitch {
                checked: !ZdlTheme.compactDensity
                Accessible.name: qsTr("Comfortable density")
                onToggled: ZdlTheme.compactDensity = !checked
            }

            Label {
                text: qsTr("Comfortable")
                font.family: ZdlTheme.fontFamily
                font.pixelSize: ZdlTheme.textCaptionSize
                color: ZdlTheme.textMuted
            }
        }
    }

    ScrollView {
        id: scroller
        anchors.fill: parent
        contentWidth: availableWidth
        clip: true

        ColumnLayout {
            width: Math.min(scroller.availableWidth, ZdlTheme.contentMaxWidth)
            spacing: ZdlTheme.space6

            ColumnLayout {
                Layout.fillWidth: true
                spacing: ZdlTheme.space2

                Label {
                    text: qsTr("Cards — rest · hover · selected · disabled · navigation")
                    font.family: ZdlTheme.fontFamily
                    font.pixelSize: ZdlTheme.textTitleLgSize
                    font.weight: Font.DemiBold
                    color: ZdlTheme.text
                }

                SettingsCard {
                    Layout.fillWidth: true
                    title: qsTr("Resting card")
                    description: qsTr("Muted surface with hairline border at elevation 1.")
                    control: ToggleSwitch {}
                }

                SettingsCard {
                    Layout.fillWidth: true
                    title: qsTr("Selected card")
                    description: qsTr("Accent wash communicates selection; focus ring is separate.")
                    selected: true
                    control: ToggleSwitch { checked: true }
                }

                SettingsCard {
                    Layout.fillWidth: true
                    title: qsTr("Disabled card")
                    description: qsTr("Full visibility, subtle foreground, reason explained here.")
                    enabled: false
                }

                SettingsCard {
                    Layout.fillWidth: true
                    title: qsTr("Navigation card")
                    description: qsTr("Ends in a chevron; activates on click, Enter, or Space.")
                    isNavigation: true
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: ZdlTheme.space2

                Label {
                    text: qsTr("Expander — one level deep")
                    font.family: ZdlTheme.fontFamily
                    font.pixelSize: ZdlTheme.textTitleLgSize
                    font.weight: Font.DemiBold
                    color: ZdlTheme.text
                }

                SettingsExpander {
                    Layout.fillWidth: true
                    title: qsTr("Advanced graphics settings")
                    description: qsTr("Contains related controls inside one expandable region.")

                    SettingsCard {
                        Layout.fillWidth: true
                        title: qsTr("Variable refresh rate")
                        control: ToggleSwitch {}
                    }

                    SettingsCard {
                        Layout.fillWidth: true
                        title: qsTr("Color profile")
                        description: qsTr("sRGB is applied while this build ships pickers.")
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: ZdlTheme.space2

                Label {
                    text: qsTr("Navigation rows")
                    font.family: ZdlTheme.fontFamily
                    font.pixelSize: ZdlTheme.textTitleLgSize
                    font.weight: Font.DemiBold
                    color: ZdlTheme.text
                }

                Repeater {
                    model: [
                        { label: qsTr("System"), badge: "" },
                        { label: qsTr("Network & internet"), badge: "3" },
                        { label: qsTr("Personalization"), badge: "" }
                    ]

                    NavRow {
                        required property var modelData
                        required property int index
                        Layout.fillWidth: true
                        label: modelData.label
                        badge: modelData.badge
                        selected: index === 0
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: ZdlTheme.space2

                Label {
                    text: qsTr("Info bars — info · warning · danger")
                    font.family: ZdlTheme.fontFamily
                    font.pixelSize: ZdlTheme.textTitleLgSize
                    font.weight: Font.DemiBold
                    color: ZdlTheme.text
                }

                InfoBar {
                    Layout.fillWidth: true
                    severity: InfoBar.Severity.Info
                    message: qsTr("System is up to date. Last checked today.")
                    actionText: qsTr("Check again")
                    onActionTriggered: root.openFeedback()
                }

                InfoBar {
                    Layout.fillWidth: true
                    severity: InfoBar.Severity.Warning
                    message: qsTr("Battery saver is limiting performance.")
                }

                InfoBar {
                    Layout.fillWidth: true
                    severity: InfoBar.Severity.Danger
                    message: qsTr("No network connection. Some pages report honest unavailable states.")
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: ZdlTheme.space2

                Label {
                    text: qsTr("Motion — press feedback · reveal · frame budget")
                    font.family: ZdlTheme.fontFamily
                    font.pixelSize: ZdlTheme.textTitleLgSize
                    font.weight: Font.DemiBold
                    color: ZdlTheme.text
                }

                SettingsCard {
                    Layout.fillWidth: true
                    title: qsTr("Frame instrumentation")
                    description: root.frameText
                    isNavigation: false

                    control: ToggleSwitch {
                        checked: ZdlTheme.debugFrames
                        Accessible.name: qsTr("Report frame statistics")
                        onToggled: ZdlTheme.debugFrames = checked
                    }
                }

                Label {
                    text: qsTr("Press and hold any card or row to feel the control spring; expanders reveal with the same spring while their height snaps — only transform and opacity animate, per the 8.3 ms frame budget.")
                    font.family: ZdlTheme.fontFamily
                    font.pixelSize: ZdlTheme.textBodySize
                    color: ZdlTheme.textMuted
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }
        }
    }

    function openFeedback() {
        infoActionFeedback.open()
        feedbackCloseTimer.restart()
    }

    readonly property string frameText: qsTr("Rolling average %1 ms · worst-in-window %2 ms · budget %3 ms")
        .arg(ZdlTheme.frameAverageMs.toFixed(2))
        .arg(ZdlTheme.frameWorstMs.toFixed(2))
        .arg(ZdlTheme.frameBudgetMs)

    Timer {
        id: feedbackCloseTimer
        interval: 2000
        onTriggered: infoActionFeedback.close()
    }

    Popup {
        id: infoActionFeedback
        parent: Overlay.overlay
        x: (parent.width - width) / 2
        y: parent.height - height - ZdlTheme.space8
        modal: false
        closePolicy: Popup.AutoClose

        background: Squircle {
            radius: ZdlTheme.radiusControl
            fillColor: ZdlTheme.surfaceElevated
            strokeColor: ZdlTheme.border
        }

        contentItem: Label {
            text: qsTr("Gallery action acknowledged.")
            font.family: ZdlTheme.fontFamily
            font.pixelSize: ZdlTheme.textBodySize
            color: ZdlTheme.text
        }
    }
}
