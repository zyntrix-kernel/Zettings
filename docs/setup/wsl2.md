# WSL2 Kubuntu setup

Zettings targets Kubuntu 24.04 LTS as its first-class Linux runtime. The
product is a native **Qt 6 (QML)** application bridged from **Rust** via
**cxx-qt** — everything must be built inside WSL2 (or any equivalent
Kubuntu 24.04 userland). The Windows host is documentation/planning only.

This doc lists every package you must install and how to run the real
application. The Zettings project never runs `apt`, `winget`, or `brew`
for you — every command here is something you execute yourself.

## Install WSL2 Kubuntu 24.04 LTS

From an Administrator PowerShell on the Windows host (run this yourself;
this command is OUTSIDE the Zettings build process):

```powershell
wsl --install -d Ubuntu-24.04
```

Enable systemd (required for the integration daemons and user services):

```bash
# /etc/wsl.conf
[boot]
systemd=true
[user]
default=<your-user>
```

Apply with `wsl --shutdown` from Windows, then reopen the distro.

## Build dependencies

Inside WSL2 (running as your normal user), install the compiler toolchain
and library headers:

```bash
sudo apt update
sudo apt install -y \
  build-essential pkg-config \
  cmake ninja-build \
  clang libclang-dev \
  libdbus-1-dev libpolkit-gobject-1-dev \
  libpulse-dev libudev-dev \
  libssl-dev
```

Install the Qt 6 development stack (base + declarative/QML + the Quick
runtime modules the UI composes from; the exact module list is finalized
in PLAN Phase 2):

```bash
sudo apt install -y \
  qt6-base-dev \
  qt6-declarative-dev \
  qml6-module-qtquick \
  qml6-module-qtquick-controls \
  qml6-module-qtquick-layouts \
  qml6-module-qtquick-templates \
  qml6-module-qtquick-window \
  qml6-module-qtqml-workerscript
```

Qt tooling (`qmllint`, `qmlformat`) ships under `/usr/lib/qt6/bin/`;
ensure it is on `PATH` for the verification gates:

```bash
export PATH="/usr/lib/qt6/bin:$PATH"
```

Rust toolchain (matches `rust-toolchain.toml`):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.97.0
source "$HOME/.cargo/env"
```

Node/pnpm are NOT required anywhere: there is no web frontend anymore.

## Verification gates (Linux target)

From the WSL2 mount of the repo:

```bash
cd "/mnt/c/Users/USER/Desktop/Zyntrix/Zyntrix OS/Zettings-app"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
```

Once QML sources exist, add the QML gate:

```bash
qmllint <changed .qml files>
qmlformat --check <changed .qml files>
```

## Run the real application under WSLg

Qt renders natively under WSLg (Wayland or XWayland); no webview
workarounds are needed.

```bash
cd "/mnt/c/Users/USER/Desktop/Zyntrix/Zyntrix OS/Zettings-app"
export QT_QPA_PLATFORM=wayland   # fall back to xcb if the compositor misbehaves
export XDG_RUNTIME_DIR=/run/user/$(id -u)
cargo run --release -p zettings
```

### Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| Window never appears after many relaunches | Wedged WSLg RemoteApp session | `wsl --shutdown` from Windows, relaunch |
| Blank/garbled rendering on Wayland | WSLg compositor quirk | `QT_QPA_PLATFORM=xcb` |
| Missing QML module error at startup | Runtime module not installed | Install the matching `qml6-module-*` package |

## Integration daemons (real backends inside WSL)

WSL cannot provide Bluetooth adapters or DRM display hardware — those
pages report honest `Unavailable` states. Everything else can run for
real:

```bash
sudo apt install -y \
  network-manager \
  power-profiles-daemon \
  pipewire pipewire-pulse wireplumber \
  pulseaudio-utils

sudo systemctl enable --now NetworkManager power-profiles-daemon
sudo loginctl enable-linger <your-user>   # start the user manager at boot
systemctl --user enable --now pipewire.socket pipewire-pulse.socket wireplumber
```

Audio flows through WSLg's PulseAudio bridge (`PULSE_SERVER` resolves to
`/mnt/wslg/PulseServer`); the `RDPSink` sink it exposes is your real
Windows audio output — volume and mute changes in Zettings are real.

## Polkit actions

Zettings ships `.policy` files declaring every
`org.zyntrix.zettings.<domain>.<verb>` action. The Linux package install
step copies them into `/usr/share/polkit-1/actions/`. The KDE polkit
agent (`polkit-kde-agent-1`) renders the auth dialog. Zettings NEVER
prompts for a password inside its own UI.

### Dev-sandbox authorization rule

WSL has no polkit authentication agent, so interactive prompts are
impossible and every privileged mutation would fail. For the sandbox
ONLY, install a scoped rule that authorizes your developer user for the
daemons Zettings integrates with:

```js
// /etc/polkit-1/rules.d/49-zettings-wsl.rules
polkit.addRule(function(action, subject) {
    if (subject.user != "<your-user>") {
        return;
    }
    var allowed = [
        "org.freedesktop.timedate1.set-ntp",
        "org.freedesktop.timedate1.set-timezone",
        "org.freedesktop.timedate1.set-time",
        "org.freedesktop.timedate1.set-local-rtc",
        "org.freedesktop.UPower.PowerProfiles.switch-profile",
        "org.freedesktop.UPower.PowerProfiles.hold-profile"
    ];
    if (allowed.indexOf(action.id) >= 0 ||
        action.id.indexOf("net.hadess.PowerProfiles") === 0) {
        return polkit.Result.YES;
    }
});
```

> NEVER copy this rule to a real system. It exists so the sandbox can
> exercise real mutations without an agent; production authorization
> always goes through the desktop agent and per-action policy.

Note: current power-profiles-daemon declares its action as
`org.freedesktop.UPower.PowerProfiles.switch-profile` (not the legacy
`net.hadess.PowerProfiles.*` ids), which is why the rule lists both.
