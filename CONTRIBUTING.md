# Contributing / Building from Source

This is the "build it by hand" doc. For how the project is organized and the conventions a change
must follow, start at [AGENTS.md](AGENTS.md) — this doc stays focused on getting a working build,
not on project philosophy or the golden rules.

## Prerequisites

| Tool | Version | Needed for |
| --- | --- | --- |
| Rust | 1.82+ (`rust-version` in [Cargo.toml](Cargo.toml)) | everything |
| Node.js | 20+ | the frontend, and any build that embeds it |
| Docker | any recent | the self-host image |
| Nix | with flakes enabled | the Nix package/dev shell |

Building the **desktop app** (`storyteller-tauri`) additionally needs Tauri v2's Linux system
dependencies and its CLI — see [Desktop app](#desktop-app-storyteller-tauri) below; nothing else in
this doc needs them.

## Rust workspace

```sh
cargo build                     # whole workspace
cargo test                      # whole workspace
cargo clippy --all-targets      # must be warning-free
cargo fmt --all
cargo run -p storyteller-server -- --project /path/to/my-novel
```

This never requires a frontend build first: `storyteller-server` embeds nothing without one
([ADR 0016](docs/adr/0016-embed-frontend-in-server-binary.md)), and `storyteller-tauri` points its
own `frontendDist` at a placeholder it never actually serves
([ADR 0019](docs/adr/0019-tauri-reuses-the-http-router.md)). `tests/fixtures/sample-project` is a
ready-made project to point the server at; it then serves `http://127.0.0.1:8787/api/v1/…`.

## Frontend

```sh
cd frontend
npm install
npm run dev        # dev server at http://localhost:5173, proxying /api to the backend above
npm run check       # svelte-check (types)
npm run build       # static build into frontend/build/
```

See [frontend/README.md](frontend/README.md) for the stack and layout in more depth.

## Self-host (Docker)

The image bundles the API and the built frontend in one binary
(`storyteller-server/src/frontend.rs`); its multi-stage `Dockerfile` builds the frontend itself,
so no local `npm run build` is needed first.

```sh
docker build -t storyteller .
docker run -p 8787:8787 -v /path/to/your/novel:/data storyteller
```

or `PROJECT_DIR=/path/to/your/novel docker compose up --build`. Open `http://localhost:8787`.

## Nix

```sh
nix build     # packages.default (storyteller-server + the built frontend)
nix run       # apps.default
```

See [flake.nix](flake.nix). As with Docker, the frontend is built as part of the Nix derivation, no
separate `npm run build` needed. Note: `frontend`'s `npmDepsHash` in the flake is a placeholder
(`pkgs.lib.fakeHash`) until someone with network access runs `nix build` once and pastes in the
real hash Nix reports — standard for a first Nix packaging pass with no network in the authoring
environment.

## Desktop app (`storyteller-tauri`)

The desktop shell runs `storyteller-server`'s own router in-process and points its window at it —
see [ADR 0019](docs/adr/0019-tauri-reuses-the-http-router.md) for why. Building it needs a couple
of one-time, Linux-specific installs beyond the base Rust/Node prerequisites:

```sh
# One-time: Tauri v2's system dependencies (Debian/Ubuntu; adjust for your distro)
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
                  librsvg2-dev patchelf build-essential

# One-time: the Tauri CLI (a real build, gives you the `cargo tauri` subcommand)
cargo install tauri-cli --version "^2.0" --locked

# The real frontend must exist before bundling — the desktop app serves it through
# storyteller-server's embed, not Tauri's own asset pipeline (ADR 0019), but the
# embed still needs `frontend/build/` on disk to pick up:
cd frontend && npm run build && cd ..
touch storyteller-server/src/frontend.rs   # force re-embedding if it was already built once

# Build and bundle
cd storyteller-tauri
cargo tauri build
```

Artifacts land under `target/release/bundle/appimage/*.AppImage` and
`target/release/bundle/deb/*.deb`. `patchelf` specifically is easy to miss — it's needed only at
AppImage-bundling time, not for a plain `cargo build`, so its absence shows up late, as a bundler
error rather than a compile error.

For local development without a full bundle, `cargo run -p storyteller-tauri` (or `cargo tauri dev`
once the CLI is installed) opens a window against a live rebuild; pass a project via the
`STORYTELLER_PROJECT` environment variable, e.g.
`STORYTELLER_PROJECT=/path/to/my-novel cargo run -p storyteller-tauri`.

## Where to go next

- [AGENTS.md](AGENTS.md) — read-first order for the specs, the golden rules, and commit
  conventions.
- [docs/architecture.md](docs/architecture.md) — how the pieces fit together.
- [docs/adr/](docs/adr/README.md) — the record of locked technical decisions; check before
  proposing something that might reopen one.
- [docs/roadmap.md](docs/roadmap.md) — what's done, what's left, and open ideas under
  investigation.
