# WSL2 Kubuntu setup

Zettings targets Kubuntu 24.04 LTS as its first-class Linux runtime. The
Windows host compiles the frontend natively; the backend crates link
against Linux system libraries, so they must be built inside WSL2
(or any equivalent Kubuntu 24.04 userland).

This doc lists every package you must install. The Zettings project never
runs `apt`, `winget`, or `brew` for you.

## Install WSL2 Kubuntu 24.04 LTS

From an Administrator PowerShell on the Windows host (run this yourself;
this command is OUTSIDE the Zettings build process):

```powershell
wsl --install -d Ubuntu-24.04
```

Inside WSL2 (running as your normal user), install the build + runtime deps:

```bash
sudo apt update
sudo apt install -y \
  build-essential pkg-config \
  libdbus-1-dev libglib2.0-dev libpolkit-gobject-1-dev \
  libpulse-dev libpipewire-0.3-dev \
  libnm-dev \
  libbluetooth-dev \
  libupower-glib-dev \
  libudev-dev libinput-dev \
  libssl-dev \
  libsbc1 \
  flatpak snapd
```

> Package notes: `libnetworkmanager-dev` and `libimage-dev` from the
> original list do not exist in Ubuntu archives — NetworkManager headers
> ship as `libnm-dev`, and no image dev package is required.

Rust toolchain (matches `rust-toolchain.toml`):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.97.0
source "$HOME/.cargo/env"
rustup target add x86_64-unknown-linux-gnu
```

Node 24 + pnpm 11 (matches `.tool-versions`):

```bash
curl -fsSL https://deb.nodesource.com/setup_24.x | sudo -E bash -
sudo apt install -y nodejs
sudo npm install -g pnpm@11.18.0
```

## Build the Zettings backend from WSL2

From the WSL2 mount of the repo:

```bash
cd "/mnt/c/Users/USER/Desktop/Zyntrix/Zyntrix OS/Zettings-app"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --target x86_64-unknown-linux-gnu
cargo nextest run --workspace
```

## Run the real backend

```bash
pnpm install --frozen-lockfile
pnpm dev:linux
```

## Polkit actions

Zettings ships `.policy` files under `apps/zettings/src-tauri/polkit/`
declaring every `org.zyntrix.zettings.<domain>.<verb>` action. The Linux
package install step copies them into `/usr/share/polkit-1/actions/`.

The KDE polkit agent (`polkit-kde-agent-1`) renders the auth dialog.
Zettings NEVER prompts for a password inside its own UI.
