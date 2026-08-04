# ADR 0009 — Rust + SvelteKit + Shadcn Stack

- **Status**: Accepted
- **Date**: 2026-08-04

## Context

The tech stack that will carry Storyteller across multiple targets needs to be fixed: self-host server, desktop app, and a mobile path (Android APK). Requirements: performance and robustness for markdown/index processing, a modern reusable web interface between desktop and self-host, reproducible packaging, and an architecture sharing maximum code between targets.

The business core (non-destructive markdown parsing, index, link resolution) demands reliability and performance: a compiled, safe language is indicated. The interface needs a productive web framework. Packaging must cover Docker, Nix, and desktop formats.

## Decision

**The chosen stack is: Rust backend, SvelteKit + Shadcn frontend, Docker/Nix packaging.**

- **Rust backend** organized as a workspace: `storyteller-core` (library, business core), `storyteller-server` (HTTP binary for self-host), `storyteller-tauri` (desktop/mobile webview). The two binaries share `storyteller-core`. See [architecture](../architecture.md).
- **SvelteKit + Shadcn frontend** for [views](../glossary.md) and editor.
- **Storage**: raw markdown = source of truth ([ADR 0002](0002-markdown-source-of-truth.md)); `cache.sqlite` index.
- **Packaging**: self-host **Docker** and **Nix**; desktop via **AppImage** / **.deb**; **Android APK** path via Tauri target.

## Consequences

- **Shared core**: `storyteller-core` centralizes logic; server and webview are just wrappers, avoiding duplication and easing the mobile target.
- **Performance and safety**: Rust suits file and index processing, with strong compile-time guarantees.
- **Single interface**: the same SvelteKit frontend serves self-host and desktop (Tauri webview), limiting UI effort.
- **Reproducibility**: Docker + Nix make builds and deployments reproducible; AppImage/.deb cover Linux desktop.
- **Mobile path open**: Tauri offers a path to Android APK, to be confirmed by a spike (see [roadmap](../roadmap.md), M7).

### Open Point — Shadcn / Tailwind

The [context](../principles.md) asks to **avoid Tailwind if possible**. Yet the Shadcn ecosystem is traditionally coupled to Tailwind. This point is **not decided** here: feasibility of Shadcn (or a Svelte variant) without Tailwind must be confirmed, or another styling approach chosen. This choice must be **re-decided in a later ADR** before GUI design. See open questions in [architecture](../architecture.md).

## Alternatives Considered

- **Backend in a managed language** (Node, Go, Python): faster to start but fewer safety/performance guarantees on file/index processing, and no webview shared path as good as Tauri for desktop/mobile. Rejected.
- **Frontend on another framework** (React, Vue): viable, but SvelteKit is retained for its lightness and ergonomics; component library choice remains conditional on the open Shadcn/Tailwind point.
- **Native app per platform**: better local integrations but UI duplication and multiplied cost; contrary to targeted code sharing. Rejected.
- **Rust + SvelteKit + Shadcn + Docker/Nix** (retained): maximum sharing via `storyteller-core`, single web interface, reproducible packaging, with the Shadcn/Tailwind adjustment to confirm.
