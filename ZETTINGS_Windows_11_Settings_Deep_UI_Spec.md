# Windows 11 Settings → ZETTINGS Deep UI & Navigation Reconstruction Specification

**Research baseline:** Windows 11 documentation and UI references available on **21 August 2026**  
**Target:** A Kubuntu-based OS named **Zyntrix OS**, with a Windows-11-inspired settings application named **ZETTINGS**  
**Purpose:** Give AI/software agents a structured, implementation-oriented specification for reconstructing the Windows Settings experience without depending on Microsoft's private source code.

---

## 0. Important scope and accuracy note

There is **no single public Microsoft repository that exposes the complete source/UI definition for every Settings page**. The most authoritative public material is split across:

1. Microsoft's `ms-settings:` URI catalog.
2. Microsoft's Windows 11 Settings overview.
3. Microsoft Fluent/WinUI design guidance.
4. The Windows 11 Settings styling/reverse-engineering community documentation, which exposes real internal XAML/control-tree paths.
5. Practical UI recreations and WinUI samples.

This document therefore separates:

- **Canonical public facts**: Microsoft-documented page names and deep links.
- **Observed implementation structure**: internal Settings control names and XAML hierarchy exposed by community tooling.
- **Design reconstruction rules**: reusable page/card/navigation patterns.
- **ZETTINGS adaptation**: how to implement the same experience using Linux/Kubuntu technologies.
- **Unknown/private areas**: items that cannot honestly be claimed as Microsoft source code.

---

# 1. Windows Settings global information architecture

Microsoft's current Windows 11 Settings overview groups the experience into these top-level areas:

- Home
- System
- Bluetooth & devices
- Network & internet
- Personalization
- Apps
- Accounts
- Time & language
- Gaming
- Accessibility
- Privacy & security
- Windows Update

Microsoft describes Home as a launch surface with interactive cards for frequently used/recommended settings; the major categories then provide the deeper configuration hierarchy.

Source:
- https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings

For ZETTINGS, preserve this basic information architecture because it is one of the most recognizable parts of the Windows Settings mental model.

---

# 2. Canonical navigation model

## 2.1 Global shell

Conceptual structure:

```text
Settings Window
│
├── Window / Title region
│
├── Navigation shell
│   ├── Back button / navigation affordance
│   ├── Navigation pane
│   │   ├── Search
│   │   ├── Home
│   │   ├── System
│   │   ├── Bluetooth & devices
│   │   ├── Network & internet
│   │   ├── Personalization
│   │   ├── Apps
│   │   ├── Accounts
│   │   ├── Time & language
│   │   ├── Gaming
│   │   ├── Accessibility
│   │   ├── Privacy & security
│   │   └── Windows Update
│   │
│   └── Footer / secondary actions where applicable
│
└── Main content viewport
    ├── Category title / breadcrumb-like heading
    ├── Intro/hero area where applicable
    ├── Settings groups
    ├── Settings rows/cards
    ├── Expanders
    ├── Links
    └── Secondary/advanced sections
```

Community reverse-engineering of Windows 11 exposes an internal hierarchy using names including:

```text
SystemSettings.View.RootPage
Microsoft.UI.Xaml.Controls.NavigationView#PermanentNavigationView
SplitView#RootSplitView
Grid#ContentRoot
Grid#ContentGrid
Frame#PermanentNavRootFrame
SystemSettings.View.CategoryPage
SystemSettings.View.L2Page#L2Page
SystemSettings.View.SettingsPageHost#pageContent
SystemSettings.View.SettingsListView#settingPagesList
SystemSettings.View.SettingsListViewItem
SystemSettings.View.EntityItem
SystemSettings.View.SettingsExpander
```

These names are useful evidence about the actual visual/control architecture, but they are **not the same thing as an open-source Microsoft implementation**.

Reference:
- https://github.com/ramensoftware/windows-11-settings-styling-guide

---

# 3. UI anatomy: the reusable primitives agents should reproduce

## 3.1 Navigation pane

Primary pattern:

```text
┌────────────────────────────────────────────────────┐
│ ☰  Search Settings...                              │
├────────────────────────────────────────────────────┤
│  ◉ Home                                             │
│  ▣ System                                           │
│  ▣ Bluetooth & devices                             │
│  ▣ Network & internet                              │
│  ▣ Personalization                                 │
│  ▣ Apps                                            │
│  ▣ Accounts                                        │
│  ▣ Time & language                                 │
│  ▣ Gaming                                          │
│  ▣ Accessibility                                   │
│  ▣ Privacy & security                              │
│  ↻ Windows Update                                  │
└────────────────────────────────────────────────────┘
```

WinUI `NavigationView` is Microsoft’s documented control for adaptive top-level navigation. It supports left, compact-left, minimal-left, and top navigation modes and can collapse/expand to preserve space.

Reference:
- https://learn.microsoft.com/en-us/windows/apps/design/controls/navigationview

### ZETTINGS rule

Use a left navigation rail/pane on desktop. At narrow widths:

```text
Wide:
[icon + text]  [content]

Medium:
[compact icons] [content]

Narrow:
[☰] [page title]
[content]
```

Do not create a giant always-open sidebar at every width.

---

# 4. Windows Settings page hierarchy model

Windows Settings can be modeled as:

```text
L0 = Application shell
L1 = Category / hub
L2 = Settings page
L3 = Section / group
L4 = Setting entity / card / row
L5 = Inline expander / sub-setting
L6 = Dialog / flyout / picker / advanced page
```

The styling-guide evidence specifically exposes both a category-page and L2-page layer:

```text
RootPage
  -> NavigationView
  -> SplitView
  -> PermanentNavRootFrame
  -> CategoryPage
  -> L2Page
  -> SettingsPageHost
  -> SettingsListView
  -> SettingsListViewItem
  -> EntityItem
```

This is extremely useful for an agent because it indicates that ZETTINGS should not treat every screen as an unrelated custom page.

Instead implement generic templates:

```text
CategoryPageTemplate
L2SettingsPageTemplate
SettingsGroupTemplate
SettingsCardTemplate
SettingsExpanderTemplate
DeviceListTemplate
DetailPageTemplate
```

---

# 5. Page layout patterns

## 5.1 Standard settings page

Recommended abstraction:

```text
Page
│
├── PageTitle
├── Optional subtitle/description
├── Optional Hero / summary
│
├── Section
│   ├── SectionHeader
│   ├── SettingCard
│   ├── SettingCard
│   └── SettingCard
│
├── Section
│   ├── SectionHeader
│   ├── SettingCard
│   └── SettingCard
│
└── About / advanced / additional section
```

Microsoft's app-settings guidance recommends a single-column, scrollable layout with constrained content width, logical section headers, and settings cards.

Reference:
- https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings

## 5.2 SettingsCard

Common structure:

```text
┌────────────────────────────────────────────────────────┐
│ [icon]  Setting title                         [control] │
│         Description / supporting text                  │
└────────────────────────────────────────────────────────┘
```

Typical controls on the right:

- Toggle switch
- Button
- Combo box
- Slider
- Hyperlink
- Secondary text
- Navigation chevron

Microsoft explicitly recommends `SettingsCard` for individual settings and `SettingsExpander` for subordinate options.

## 5.3 SettingsExpander

Pattern:

```text
┌────────────────────────────────────────────────────────┐
│ Advanced settings                              v       │
├────────────────────────────────────────────────────────┤
│ Sub-setting                                             │
│ Sub-setting                                             │
│ Sub-setting                                             │
└────────────────────────────────────────────────────────┘
```

Use one expansion level rather than deeply nested accordions.

---

# 6. Core visual language

## 6.1 Typography

Windows 11 uses **Segoe UI Variable** as the system UI font. Microsoft documents regular and semibold as the primary UI weights and recommends minimum text sizes around 12px regular and 14px semibold.

Useful reconstruction hierarchy:

```text
Page title       ~28–32px semibold
Section heading  ~14–16px semibold
Card title       ~14px / 15px
Body             ~14px regular
Secondary        ~12–13px regular
```

These values are a reconstruction baseline rather than a claim that every Settings page uses identical pixel values.

Reference:
- https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography

## 6.2 Geometry

Windows 11 uses rounded geometry. Microsoft documents default control corner radius around 4px and overlay radius around 8px in the Fluent/WinUI system, with progressive rounding used throughout the language.

Reference:
- https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/geometry

For ZETTINGS:

```text
Small control radius      4–6 px
Cards / medium containers 8–12 px
Large hero surfaces       12–16 px
Modal / overlay           8–12 px
```

Treat these as a design system, not hard-coded one-off values.

## 6.3 Materials

Windows uses:

- Mica for long-lived window/background surfaces.
- Acrylic for transient surfaces such as menus/flyouts.

Reference:
- https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic
- https://learn.microsoft.com/en-us/windows/apps/get-started/best-practices

Kubuntu equivalent:

```text
Long-lived:
  KWin-compatible translucent/blurred background where available

Transient:
  Plasma/KWin blur/acrylic-like surface

Fallback:
  Opaque theme color when compositing is disabled
```

## 6.4 Motion

Microsoft characterizes Windows motion as reactive, direct, and context-appropriate. Use motion to explain:

- navigation
- expansion
- control state changes
- overlays
- selection
- hierarchy changes

Reference:
- https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/motion

Avoid gratuitous animations that slow settings workflows.

---

# 7. Interaction/control inventory

Agents should implement reusable controls:

```text
ToggleSwitch
CheckBox
RadioButton
ComboBox
Slider
TextBox
PasswordBox
Button
Hyperlink
ListView
Data grid/list
Expander
SettingsCard
SettingsExpander
InfoBar
ProgressBar
ProgressRing
Calendar/DatePicker
TimePicker
ColorPicker
Flyout
Menu
ContextMenu
SearchBox / AutoSuggestBox
Device picker
File picker
Permission list
Navigation row
```

Microsoft's current control documentation catalogs these as standard Windows app controls.

Reference:
- https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/

---

# 8. Search architecture

## 8.1 Basic Settings search

The Settings search experience should be modeled as:

```text
User query
   ↓
Normalization
   ↓
Keyword / synonym matching
   ↓
Settings page index
   ↓
Matching setting
   ↓
Category
   ↓
Deep navigation target
```

Microsoft documents Settings search pages including:

```text
ms-settings:search
ms-settings:search-moredetails
ms-settings:search-permissions
```

## 8.2 Search result model

Every indexed setting should have:

```json
{
  "id": "display.scale",
  "title": "Scale",
  "description": "Change the size of text, apps, and other items",
  "category": "System",
  "page": "Display",
  "route": "/system/display",
  "aliases": [
    "text size",
    "UI scale",
    "make things bigger",
    "make things smaller"
  ],
  "keywords": [
    "scaling",
    "DPI",
    "screen size"
  ],
  "controlType": "combo",
  "action": "open-control"
}
```

## 8.3 Natural-language search

Current Windows 11 documentation describes an on-device Settings AI agent for supported Copilot+ PCs. Microsoft says it can interpret intent and suggest settings; when confidence is insufficient, normal search results are shown, and the user must explicitly apply changes.

Reference:
- https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings

For ZETTINGS, this is an excellent design:

```text
Query
  ↓
Exact keyword search
  ↓ if weak
Semantic search
  ↓ if high confidence
Intent explanation
  ↓
Candidate setting(s)
  ↓
Preview
  ↓
Apply
```

Never allow an AI agent to silently mutate arbitrary system configuration.

---

# 9. Search result ranking recommended for ZETTINGS

Use a weighted ranking:

```text
exact title match        +100
exact keyword match       +80
alias match               +70
page/category match       +50
description match         +35
semantic similarity       +30
recently used             +10
fuzzy spelling             +5
```

Then boost direct settings above general category pages.

Example:

Query: "make my mouse pointer bigger"

Preferred:

```text
Mouse pointer size
  Accessibility → Mouse pointer and touch
```

not merely:

```text
Accessibility
```

---

# 10. Complete public ms-settings deep-link catalog

The authoritative Microsoft catalog currently documents the following groups and links. Availability varies by Windows version, hardware, installed components, and SKU.

## Accounts

```text
Access work or school
ms-settings:workplace

Email & app accounts
ms-settings:emailandaccounts

Family & other people
ms-settings:otherusers

Provisioning
ms-settings:provisioning

Workplace provisioning
ms-settings:workplace-provisioning

Repair token
ms-settings:workplace-repairtoken

Set up a kiosk
ms-settings:assignedaccess

Sign-in options
ms-settings:signinoptions

Dynamic lock
ms-settings:signinoptions-dynamiclock

Sync your settings
ms-settings:sync

Backup
ms-settings:backup

Windows Anywhere
ms-settings:windowsanywhere

Windows Hello face enrollment
ms-settings:signinoptions-launchfaceenrollment

Windows Hello fingerprint enrollment
ms-settings:signinoptions-launchfingerprintenrollment

Your info
ms-settings:yourinfo
```

## Apps

```text
Apps & Features
ms-settings:appsfeatures

Specific app features
ms-settings:appsfeatures-app?<PACKAGE-FAMILY-NAME>

Apps for websites
ms-settings:appsforwebsites

Default apps
ms-settings:defaultapps

Default browser settings
ms-settings:defaultbrowsersettings

Optional features
ms-settings:optionalfeatures

Offline Maps
ms-settings:maps

Download maps
ms-settings:maps-downloadmaps

Startup apps
ms-settings:startupapps

Video playback
ms-settings:videoplayback
```

`defaultapps` can accept parameters that target a registered application. Microsoft documents parameters including registered application name and AUMID.

## Control Center

```text
Control center
ms-settings:controlcenter
```

## Bluetooth & devices / Devices

```text
AutoPlay
ms-settings:autoplay

Bluetooth
ms-settings:bluetooth

Connected Devices
ms-settings:connecteddevices

Camera
ms-settings:camera

Mouse & touchpad
ms-settings:mousetouchpad

Pen & Windows Ink
ms-settings:pen

Pen shortcut button
ms-settings:pen-button

Printers & scanners
ms-settings:printers

Touch
ms-settings:devices-touch

Touchpad
ms-settings:devices-touchpad

Text Suggestions
ms-settings:devicestyping-hwkbtextsuggestions

Typing
ms-settings:typing

USB
ms-settings:usb

Wheel
ms-settings:wheel

Your phone / Mobile devices
ms-settings:mobile-devices
```

## Accessibility / Ease of access

```text
Audio
ms-settings:easeofaccess-audio

Closed captions
ms-settings:easeofaccess-closedcaptioning

Color filters
ms-settings:easeofaccess-colorfilter

Adaptive color filter link
ms-settings:easeofaccess-colorfilter-adaptivecolorlink

Blue-light link
ms-settings:easeofaccess-colorfilter-bluelightlink

Display
ms-settings:easeofaccess-display

Eye control
ms-settings:easeofaccess-eyecontrol

Hearing devices
ms-settings:easeofaccess-hearingaids

High contrast
ms-settings:easeofaccess-highcontrast

Keyboard
ms-settings:easeofaccess-keyboard

Magnifier
ms-settings:easeofaccess-magnifier

Mouse
ms-settings:easeofaccess-mouse

Mouse pointer & touch
ms-settings:easeofaccess-mousepointer

Narrator
ms-settings:easeofaccess-narrator

Narrator autostart
ms-settings:easeofaccess-narrator-isautostartenabled

Speech
ms-settings:easeofaccess-speechrecognition

Text cursor
ms-settings:easeofaccess-cursor

Visual Effects
ms-settings:easeofaccess-visualeffects
```

## Extras

```text
Extras
ms-settings:extras
```

This is an extension point for applications that install their own settings.

## Family Group

```text
Family Group
ms-settings:family-group
```

## Gaming

```text
Game bar
ms-settings:gaming-gamebar

Game DVR
ms-settings:gaming-gamedvr

Game Mode
ms-settings:gaming-gamemode

Playing a game full screen
ms-settings:quietmomentsgame
```

## Holographic / mixed-reality related

```text
Audio and speech
ms-settings:holographic-audio

Environment
ms-settings:privacy-holographic-environment

Headset display
ms-settings:holographic-headset

Uninstall
ms-settings:holographic-management

Startup and desktop
ms-settings:holographic-startupandesktop
```

## Network & internet

```text
Network & internet
ms-settings:network-status

Advanced network settings
ms-settings:network-advancedsettings

Airplane mode
ms-settings:network-airplanemode

Proximity
ms-settings:proximity

Cellular & SIM
ms-settings:network-cellular

Dial-up
ms-settings:network-dialup

DirectAccess
ms-settings:network-directaccess

Ethernet
ms-settings:network-ethernet

Manage known networks
ms-settings:network-wifisettings

Mobile hotspot
ms-settings:network-mobilehotspot

Proxy
ms-settings:network-proxy

VPN
ms-settings:network-vpn

Wi-Fi
ms-settings:network-wifi

Wi-Fi provisioning
ms-settings:wifi-provisioning
```

## Personalization

```text
Background
ms-settings:personalization-background

Start folders
ms-settings:personalization-start-places

Colors
ms-settings:personalization-colors
ms-settings:colors

Copilot key customization
ms-settings:personalization-textinput-copilot-hardwarekey

Dynamic Lighting
ms-settings:personalization-lighting

Fonts
ms-settings:fonts

Glance
ms-settings:personalization-glance

Lock screen
ms-settings:lockscreen

Navigation bar
ms-settings:personalization-navbar

Personalization category
ms-settings:personalization

Start
ms-settings:personalization-start

Taskbar
ms-settings:taskbar

Text input
ms-settings:personalization-textinput

Touch keyboard
ms-settings:personalization-touchkeyboard

Themes
ms-settings:themes
```

## Phone

```text
Phone Link
ms-settings:mobile-devices

Add phone
ms-settings:mobile-devices-addphone

Add phone direct
ms-settings:mobile-devices-addphone-direct

Device Usage
ms-settings:deviceusage
```

## Privacy

```text
Accessory apps
ms-settings:privacy-accessoryapps

Account info
ms-settings:privacy-accountinfo

Activity history
ms-settings:privacy-activityhistory

Advertising ID
ms-settings:privacy-advertisingid

App diagnostics
ms-settings:privacy-appdiagnostics

Automatic file downloads
ms-settings:privacy-automaticfiledownloads

Background apps
ms-settings:privacy-backgroundapps

Background Spatial Perception
ms-settings:privacy-backgroundspatialperception

Calendar
ms-settings:privacy-calendar

Call history
ms-settings:privacy-callhistory

Camera
ms-settings:privacy-webcam

Contacts
ms-settings:privacy-contacts

Documents
ms-settings:privacy-documents

Downloads folder
ms-settings:privacy-downloadsfolder

Email
ms-settings:privacy-email

Eye tracker
ms-settings:privacy-eyetracker

Feedback & diagnostics
ms-settings:privacy-feedback

File system
ms-settings:privacy-broadfilesystemaccess

General
ms-settings:privacy
ms-settings:privacy-general

Graphics
ms-settings:privacy-graphicscaptureprogrammatic
ms-settings:privacy-graphicscapturewithoutborder

Inking & typing
ms-settings:privacy-speechtyping

Location
ms-settings:privacy-location

Messaging
ms-settings:privacy-messaging

Microphone
ms-settings:privacy-microphone

Motion
ms-settings:privacy-motion

Music Library
ms-settings:privacy-musiclibrary

Notifications
ms-settings:privacy-notifications

Other devices
ms-settings:privacy-customdevices

Phone calls
ms-settings:privacy-phonecalls

Pictures
ms-settings:privacy-pictures

Radios
ms-settings:privacy-radios

Speech
ms-settings:privacy-speech

Tasks
ms-settings:privacy-tasks

Videos
ms-settings:privacy-videos

Voice activation
ms-settings:privacy-voiceactivation
```

## Search

```text
Search
ms-settings:search

Search more details
ms-settings:search-moredetails

Search permissions
ms-settings:search-permissions
```

## Sound

```text
Volume mixer
ms-settings:apps-volume

Sound
ms-settings:sound

Sound devices
ms-settings:sound-devices

Default microphone
ms-settings:sound-defaultinputproperties

Default audio output
ms-settings:sound-defaultoutputproperties

Specific audio endpoint
ms-settings:sound-properties?endpointId=<ENDPOINT_ID>

Specific audio interface
ms-settings:sound-properties?interfaceId=<INTERFACE_ID>
```

## System

```text
About
ms-settings:about

Advanced display
ms-settings:display-advanced

Battery Saver
ms-settings:batterysaver

Battery Saver settings
ms-settings:batterysaver-settings

Battery use
ms-settings:batterysaver-usagedetails

Clipboard
ms-settings:clipboard

Default save locations
ms-settings:savelocations

Display
ms-settings:display

Screen rotation
ms-settings:screenrotation

Duplicate displays
ms-settings:quietmomentspresentation

During these hours
ms-settings:quietmomentsscheduled

Encryption
ms-settings:deviceencryption

Energy recommendations
ms-settings:energyrecommendations

Focus assist
ms-settings:quiethours

Graphics settings
ms-settings:display-advancedgraphics

Graphics default settings
ms-settings:display-advancedgraphics-default

Multitasking
ms-settings:multitasking

Multitasking SG update
ms-settings:multitasking-sgupdate

Night light
ms-settings:nightlight

Projecting to this PC
ms-settings:project

Shared experiences
ms-settings:crossdevice

Tablet mode
ms-settings:tabletmode

Taskbar
ms-settings:taskbar

Notifications
ms-settings:notifications

Remote Desktop
ms-settings:remotedesktop

Phone
ms-settings:phone

Power & sleep
ms-settings:powersleep

Presence sensing
ms-settings:presence

Storage
ms-settings:storagesense

Storage Sense
ms-settings:storagepolicies

Storage recommendations
ms-settings:storagerecommendations

Disks & volumes
ms-settings:disksandvolumes
```

## Time & language

```text
Date & time
ms-settings:dateandtime

Japan IME
ms-settings:regionlanguage-jpnime

Region
ms-settings:regionformatting

Keyboard / language
ms-settings:keyboard
ms-settings:keyboard-advanced
ms-settings:regionlanguage

BPMF IME
ms-settings:regionlanguage-bpmfime

Cangjie IME
ms-settings:regionlanguage-cangjieime

Wubi UDP / related IME
ms-settings:regionlanguage-chsime-wubi-udp

Quick IME
ms-settings:regionlanguage-quickime

Korean IME
ms-settings:regionlanguage-korime

Pinyin IME
ms-settings:regionlanguage-chsime-pinyin

Pinyin domain lexicon
ms-settings:regionlanguage-chsime-pinyin-domainlexicon

Pinyin key config
ms-settings:regionlanguage-chsime-pinyin-keyconfig

Pinyin UDP
ms-settings:regionlanguage-chsime-pinyin-udp

Speech
ms-settings:speech

Wubi IME
ms-settings:regionlanguage-chsime-wubi
```

## Update & security

```text
Activation
ms-settings:activation

Backup
ms-settings:backup

Delivery Optimization
ms-settings:delivery-optimization

Delivery Optimization activity
ms-settings:delivery-optimization-activity

Delivery Optimization advanced
ms-settings:delivery-optimization-advanced

Find My Device
ms-settings:findmydevice

For developers
ms-settings:developers

Recovery
ms-settings:recovery

Security key enrollment
ms-settings:signinoptions-launchsecuritykeyenrollment

Troubleshoot
ms-settings:troubleshoot

Windows Security
ms-settings:windowsdefender

Windows Insider Program
ms-settings:windowsinsider

Windows Insider opt-in
ms-settings:windowsinsider-optin

Windows Update
ms-settings:windowsupdate

Windows Update action
ms-settings:windowsupdate-action

Active hours
ms-settings:windowsupdate-activehours

Advanced options
ms-settings:windowsupdate-options

Optional updates
ms-settings:windowsupdate-optionalupdates

Restart options
ms-settings:windowsupdate-restartoptions

Seeker / on-demand update
ms-settings:windowsupdate-seekerondemand

Update history
ms-settings:windowsupdate-history
```

Microsoft explicitly flags a number of older/deprecated/conditional entries. Do not blindly recreate every historical URI as a visible ZETTINGS page.

---

# 11. Deep-link design for ZETTINGS

Windows uses URI-addressable settings destinations.

ZETTINGS should use its own route scheme instead of copying `ms-settings:`.

Recommended:

```text
zettings://system
zettings://system/display
zettings://system/sound
zettings://system/power
zettings://devices/bluetooth
zettings://devices/printers
zettings://network/wifi
zettings://network/ethernet
zettings://personalization/background
zettings://personalization/themes
zettings://apps/installed
zettings://accounts/sign-in
zettings://accessibility/display
zettings://privacy/microphone
zettings://updates
```

Internal application routing can then map these to Qt/QML/Tauri routes.

---

# 12. Route schema for AI agents

Use a machine-readable registry.

```json
{
  "id": "system.display",
  "title": "Display",
  "category": "System",
  "route": "/system/display",
  "icon": "display",
  "description": "Configure displays, scaling, resolution, HDR and related options.",
  "aliases": [
    "monitor",
    "screen",
    "resolution",
    "scaling",
    "brightness"
  ],
  "children": [
    "system.display.scale",
    "system.display.resolution",
    "system.display.brightness",
    "system.display.hdr",
    "system.display.night-light"
  ],
  "platform": {
    "linux": [
      "KScreen",
      "Wayland",
      "XRandR-compatible fallback"
    ]
  }
}
```

Every setting should have:

```text
ID
Category
Page
Section
Title
Description
Aliases
Keywords
Icon
Control type
Current value provider
Set-value action
Validation
Requires admin
Requires hardware
Requires reboot
Route
Search weight
```

---

# 13. Windows → Kubuntu functional mapping

## System

```text
Windows Display              → KScreen / KWin display APIs
Windows Sound                → PipeWire / WirePlumber
Windows Network              → NetworkManager
Windows Bluetooth            → BlueZ
Windows Power                → power-profiles-daemon / UPower
Windows Storage              → udisks2 / filesystem APIs
Windows Users                → AccountsService / system user tools
Windows Updates              → distro/package/update backend
Windows Firewall             → ufw / firewalld backend
Windows Accessibility        → KDE accessibility + AT-SPI
Windows Privacy permissions  → KDE/Linux permission model where available
Windows Default Apps         → XDG MIME/default application settings
Windows Printers              → CUPS
Windows Fonts                → fontconfig/KDE font management
Windows Time                  → systemd-timesyncd / chrony / timedatectl
Windows Network VPN          → NetworkManager VPN plugins
Windows Disk management      → udisks2/KDE partition tools
```

Do not force a fake Windows backend onto Linux. Reproduce the **user model**, while using native Linux APIs underneath.

---

# 14. ZETTINGS page templates

Implement these first:

### Template A — Category hub

```text
Page title
Short description

[Card]
  title
  description
  optional status/value

[Card]
  title
  description
  optional status/value

[Section]
  Setting rows...
```

### Template B — Standard settings list

```text
Back / breadcrumb
Page title
Subtitle

Section title
[SettingsCard]
[SettingsCard]
[SettingsCard]

Section title
[SettingsCard]
[SettingsCard]
```

### Template C — Device manager page

```text
Page title

Status / overview card

Devices
[device row + status]
[device row + status]
[device row + status]

Actions
[Add device]
```

### Template D — Detail page

```text
Back
Icon
Title
Description

Main setting
Secondary setting
Advanced expander

Related settings
```

### Template E — Search result

```text
Search query
────────────────────────────
Settings result
  Title
  Description
  Category > Page
  [Open]
```

---

# 15. Loading and state behavior

Agents should reproduce system-app behaviors:

```text
Loading:
  skeleton/placeholder when a page depends on slow system state.

Disabled:
  preserve visibility when possible;
  explain why disabled.

Permission:
  show explanation + action.

No device:
  empty-state card.

No network:
  stateful offline UI.

Error:
  human-readable message + retry.

Success:
  update the value immediately where possible.

Unsaved:
  only use explicit Apply/Save when necessary.
```

For simple settings, Microsoft recommends immediate reflection of changes rather than requiring unnecessary confirmation dialogs.

---

# 16. Responsiveness

Use breakpoints based on available content width rather than a fixed device type:

```text
> 1100 px
  full navigation + spacious content

~ 800–1100 px
  compact navigation + content

~ 560–800 px
  minimal navigation / overlay navigation

< 560 px
  mobile-style single-column layout
```

Content itself should stay constrained so very wide monitors do not create extremely long text lines.

Microsoft's settings guidance recommends a scrollable layout with a constrained maximum content width around 1000–1100 px for settings pages.

---

# 17. Accessibility requirements

ZETTINGS should reproduce the functional advantages of Settings, not just its appearance.

Every control must have:

```text
Accessible name
Accessible description where necessary
Keyboard focus
Visible focus indicator
Logical tab order
Screen-reader role
State reporting
Keyboard activation
High-contrast compatible styling
Reduced-motion behavior
```

Minimum keyboard flows:

```text
Ctrl/Cmd+F or dedicated search shortcut → search
Alt/Backspace or app back               → previous page
Arrow keys                              → list navigation
Enter/Space                             → activate
Escape                                  → dismiss overlay
Tab / Shift+Tab                         → focus traversal
```

---

# 18. Context navigation and breadcrumbs

Page hierarchy should remain recoverable.

Example:

```text
System
  > Display
    > Advanced display
```

A row that navigates deeper should make the destination obvious:

```text
HDR
Turn on HDR for supported displays                       >
```

Do not force users to guess whether a row is a toggle or a navigation link.

---

# 19. Data-driven page architecture

Avoid writing each page as an isolated hard-coded component.

Preferred model:

```text
Registry
   ↓
Page definition
   ↓
Section definition
   ↓
Setting definition
   ↓
Control renderer
   ↓
Native Linux backend
```

Example:

```json
{
  "page": "system.power",
  "title": "Power",
  "sections": [
    {
      "title": "Power mode",
      "items": [
        {
          "id": "power.profile",
          "type": "enum",
          "backend": "power_profiles_daemon",
          "options": [
            "power-saver",
            "balanced",
            "performance"
          ]
        }
      ]
    }
  ]
}
```

This is especially important for AI-assisted development.

---

# 20. AI-agent task decomposition

Give each coding agent one of these roles:

```text
Agent 1 — Information architecture
Agent 2 — Navigation shell
Agent 3 — Design tokens
Agent 4 — Typography
Agent 5 — Settings cards
Agent 6 — Search indexing/ranking
Agent 7 — Route/deep-link registry
Agent 8 — System backend adapters
Agent 9 — Accessibility
Agent 10 — Animation/motion
Agent 11 — Theming / dark mode
Agent 12 — QA / screenshot comparison
```

Each agent should consume the same canonical registry.

---

# 21. Suggested repository structure

```text
zettings/
├── app/
│   ├── shell/
│   ├── navigation/
│   ├── search/
│   ├── routing/
│   └── window/
│
├── design/
│   ├── tokens/
│   ├── typography/
│   ├── icons/
│   ├── geometry/
│   ├── motion/
│   └── materials/
│
├── pages/
│   ├── home/
│   ├── system/
│   ├── devices/
│   ├── network/
│   ├── personalization/
│   ├── apps/
│   ├── accounts/
│   ├── time-language/
│   ├── gaming/
│   ├── accessibility/
│   ├── privacy/
│   └── updates/
│
├── components/
│   ├── settings-card/
│   ├── settings-expander/
│   ├── navigation-row/
│   ├── device-card/
│   ├── info-bar/
│   ├── picker/
│   ├── slider/
│   └── toggle/
│
├── backends/
│   ├── networkmanager/
│   ├── pipewire/
│   ├── bluez/
│   ├── power/
│   ├── udisks/
│   ├── cups/
│   ├── accounts/
│   └── updates/
│
└── registry/
    ├── pages.json
    ├── settings.json
    ├── aliases.json
    └── routes.json
```

---

# 22. Recommended implementation strategy for Kubuntu

For a ZETTINGS application that wants Windows-like visuals while retaining strong Linux integration:

## Option A — Tauri + web UI

```text
Frontend:
React / Solid / Svelte

Visual system:
CSS design tokens
SVG icons
Blur/transparency effects

Backend:
Tauri + Rust

System integration:
D-Bus
NetworkManager
PipeWire
BlueZ
UPower
UDisks2
AccountsService
CUPS
systemd interfaces
```

## Option B — Qt/QML

For a more deeply native KDE implementation:

```text
Qt/QML
KDE Frameworks
KConfig
KIO
KWindowSystem
Solid
NetworkManagerQt
BluezQt
PulseAudio/PipeWire integrations
D-Bus
```

For a Kubuntu-first operating system, Qt/QML is arguably the closer architectural fit to KDE; Tauri is attractive if your team/agents are much stronger in web UI.

---

# 23. Visual reverse-engineering methodology

Do not instruct agents to copy screenshots pixel-for-pixel first.

Use this order:

```text
1. Identify information hierarchy
2. Identify reusable page templates
3. Identify control types
4. Identify typography
5. Identify spacing
6. Identify geometry
7. Identify icons
8. Identify materials
9. Identify motion
10. Implement responsive behavior
11. Connect native backend
12. Screenshot compare
13. Iterate
```

The Windows 11 styling guide is especially useful because it exposes real internal target names such as:

```text
NavigationView#PermanentNavigationView
SplitView#RootSplitView
Grid#ContentRoot
Grid#ContentGrid
SystemSettings.View.L2Page
SystemSettings.View.SettingsPageHost
SystemSettings.View.SettingsListView
SystemSettings.View.SettingsListViewItem
SystemSettings.View.EntityItem
SystemSettings.View.SettingsExpander
```

That gives agents a much better structural clue than screenshots alone.

---

# 24. What is NOT publicly documented

Do not tell an AI agent that any of the following has been fully reverse-engineered from public source unless you actually possess evidence:

- complete private Settings source code
- every internal business-logic class
- every private XAML resource
- every internal search ranking formula
- every proprietary machine-learning model
- every hidden/experimental page
- all server-side Windows configuration behavior
- exact current pixel measurements for every individual page on every build

Instead mark such areas:

```text
STATUS: observed / inferred / public / unknown
```

Example:

```json
{
  "feature": "Settings search ranking",
  "status": "partially-public",
  "publicEvidence": [
    "documented search routes",
    "documented Settings AI behavior"
  ],
  "privateDetails": "unknown"
}
```

---

# 25. Recommended ZETTINGS master specification schema

Every page should ultimately be represented like this:

```json
{
  "id": "system.display",
  "windowsReference": {
    "category": "System",
    "page": "Display",
    "uri": "ms-settings:display"
  },
  "zettings": {
    "route": "/system/display",
    "title": "Display",
    "icon": "display"
  },
  "layout": {
    "template": "standard-settings-page",
    "maxContentWidth": 1100,
    "scrollable": true,
    "sections": [
      "display",
      "brightness",
      "color",
      "scale",
      "multiple-displays",
      "advanced"
    ]
  },
  "search": {
    "aliases": [
      "monitor",
      "screen",
      "resolution",
      "scaling"
    ]
  },
  "backend": {
    "primary": "kscreen"
  }
}
```

---

# 26. Agent rules

Use these rules in every ZETTINGS implementation prompt:

```text
RULE 1:
Never create an isolated settings page when an existing template applies.

RULE 2:
All settings must have stable IDs.

RULE 3:
All settings must be searchable by title, description, aliases and keywords.

RULE 4:
Every navigable setting must have a route.

RULE 5:
Every route must map to exactly one page definition.

RULE 6:
The UI layer must not directly manipulate Linux system files.

RULE 7:
System changes must go through a backend adapter.

RULE 8:
Privileged operations must be isolated and validated.

RULE 9:
The UI must remain functional when an optional backend is unavailable.

RULE 10:
The visual layer should emulate the Windows mental model, not pretend to be Microsoft's proprietary source code.
```

---

# 27. Primary authoritative references

## Microsoft

### Windows Settings overview
https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings

Use for:
- top-level information architecture
- current Settings categories
- current Home experience
- current Settings search/AI behavior

### `ms-settings:` URI reference
https://learn.microsoft.com/en-us/windows/apps/develop/launch/launch-settings

Use for:
- official public deep links
- supported page names
- parameters
- version/hardware caveats

### WinUI NavigationView
https://learn.microsoft.com/en-us/windows/apps/design/controls/navigationview

Use for:
- navigation pane behavior
- compact/minimal/expanded modes
- hierarchical navigation
- content/header geometry

### Settings page guidelines
https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings

Use for:
- settings cards
- expanders
- content width
- section organization
- immediate-change behavior

### Typography
https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography

Use for:
- Segoe UI Variable
- weight hierarchy
- minimum type sizes

### Geometry
https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/geometry

Use for:
- corner radii
- spacing/shape principles

### Acrylic
https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic

Use for:
- translucent materials
- transient surfaces
- accessibility/legibility cautions

### Motion
https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/motion

Use for:
- transition principles

### Controls
https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/

Use for:
- control inventory and expected interaction patterns

### Windows design resources
https://learn.microsoft.com/en-us/windows/apps/design/downloads/

Use for:
- Windows Design Kit
- WinUI Gallery
- recommended fonts/icons

## Community reverse-engineering evidence

### Windows 11 Settings Styling Guide
https://github.com/ramensoftware/windows-11-settings-styling-guide

Use for:
- internal control-tree names
- Settings root/page/control hierarchy
- actual target paths used by Windows 11 Settings styling tools

---

# 28. Final reconstruction target

The ideal ZETTINGS experience should feel like:

```text
Windows 11 Settings information architecture
            +
Windows/Fluent visual grammar
            +
KDE/Linux-native implementation
            +
Data-driven settings registry
            +
Strong search
            +
Stable deep links
            +
Native backend adapters
            +
Accessible keyboard/screen-reader behavior
            +
Responsive navigation
```

The objective is **not** to reproduce Microsoft's proprietary implementation internally.

The objective is to reproduce the **observable product architecture and user experience** while making every actual configuration operation native to Kubuntu/Zyntrix OS.