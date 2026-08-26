# WSL2 Kubuntu setup

Zettings targets Kubuntu 24.04 LTS as its first-class Linux runtime. The
Windows host compiles the frontend natively; the backend crates link
against Linux system libraries, so they must be built inside WSL2
(or any equivalent Kubuntu 24.04 userland).

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

Inside WSL2 (running as your normal user), install the build + gate deps:

```bash
sudo apt update
sudo apt install -y \
  build-essential pkg-config \
  libdbus-1-dev libpolkit-gobject-1-dev \
  libpulse-dev libudev-dev \
  libssl-dev
```

Install the Tauri v2 runtime prerequisites (the same set CI installs —
see `.github/workflows/ci.yml`):

```bash
sudo apt install -y --no-install-recommends \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libxdo-dev
```

Rust toolchain (matches `rust-toolchain.toml`):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.97.0
source "$HOME/.cargo/env"
```

Node/pnpm are NOT required inside WSL: the frontend is built on the
Windows host (`pnpm -F zettings-web build`) and the produced
`apps/zettings/web/dist` is embedded into the Linux binary at compile
time. Only install Node inside WSL if you specifically want to run the
web toolchain there.

## Verification gates (Linux target)

From the WSL2 mount of the repo:

```bash
cd "/mnt/c/Users/USER/Desktop/Zyntrix/Zyntrix OS/Zettings-app"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo test -p zettings-ipc --features export-bindings
git diff --exit-code -- packages/ts-bindings/src/generated
```

Frontend gates run on the Windows host: `pnpm -r typecheck` and
`pnpm -F zettings-web test`.

## Run the real application under WSLg

Build the frontend on the Windows host first:

```powershell
pnpm -F zettings-web build
```

Then inside WSL2:

```bash
cd "/mnt/c/Users/USER/Desktop/Zyntrix/Zyntrix OS/Zettings-app"
cargo run --release -p zettings --features custom-protocol
```

`custom-protocol` is mandatory for embedded-asset runs: tauri-macros
serves `build.devUrl` whenever the feature is off, regardless of
`--release`, which fails with `Could not connect to localhost` unless a
Vite dev server is listening on port 1420. (Hot-reload alternative: run
`pnpm -F zettings-web dev` on the Windows host and a plain
`cargo run -p zettings` debug build in WSL — localhost is shared.)

Required environment for WSLg (export before `cargo run`):

```bash
export DISPLAY=:0
export GDK_BACKEND=x11                 # XWayland path is the stable one
export XDG_RUNTIME_DIR=/run/user/$(id -u)
# WebKitGTK has no GPU under WSLg; the DMABUF renderer yields a blank
# window. Force software rendering:
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export LIBGL_ALWAYS_SOFTWARE=1
```

`libEGL`/`MESA/ZINK` warnings in the output are cosmetic; the webview
falls back to llvmpipe.

### Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| `Could not connect to localhost` | Ran without `custom-protocol`, no dev server | Add the feature or start Vite on :1420 |
| Window opens but is blank | WebKit DMABUF renderer under WSLg | The three `WEBKIT_*`/`LIBGL_*` exports above |
| Window registered but never appears (weston log shows `appId:zettings`) | Wedged WSLg RemoteApp session after many relaunches | `wsl --shutdown` from Windows, relaunch |
| `frontendDist ... doesn't exist` at compile | `web/dist` not built | Run `pnpm -F zettings-web build` on Windows |

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
