# ADR 0003 — Single-User, Local / Self-Host

- **Status**: Accepted
- **Date**: 2026-08-04

## Context

Storyteller targets an author organizing their novel's context. A decision is needed on the usage model: **local single-user** software, or **multi-user** service with accounts, authentication, and collaboration.

Multi-user triggers a cascade of complexity: authentication, account management, permissions and sharing, concurrent edit conflict resolution, secrets management, network attack surface, compliance, and service operations. This complexity brings nothing to the solo author, who is the product's target.

## Decision

**Storyteller is single-user, local or self-hosted.**

- **No authentication**, no accounts, no real-time collaboration, no permissions or secrets management.
- The tool runs on the author's machine (desktop/mobile application) or on a server they host themselves for their own use.
- The Rust workspace distinguishes `storyteller-server` (HTTP binary for self-host) and `storyteller-tauri` (desktop/mobile webview), which share `storyteller-core`. See [architecture](../architecture.md).

## Consequences

- **Radical simplicity**: no auth layer, no permissions model, no session management. Attack surface and maintenance burden are minimal.
- **Clear mental model**: one user, their files, their machine. Consistent with markdown source of truth ([ADR 0002](0002-markdown-source-of-truth.md)) and one folder per project ([ADR 0004](0004-one-folder-per-project.md)).
- **Exposure responsibility on user**: in self-host, it's up to the author not to expose the service on an untrusted network, since there's no auth. [Usage](../usage.md) documentation must remind them.
- **Collaboration = out of application**: sharing happens via external means (Git, shared folder), with no merge tooling guaranteed by Storyteller.
- Accepted consequence: the product does not target teams or SaaS use cases. Such a need would require a major overhaul and a new ADR.

## Alternatives Considered

- **Multi-user server with auth and permissions**: necessary for collaboration, but complexity and risk surface disproportionate for solo author usage. Rejected.
- **Hosted SaaS**: adds accounts, billing, hosting others' data, compliance, and operations costs; contrary to local / self-host principle and markdown source of truth. Rejected.
- **Local single-user with self-host option** (retained): covers desktop, mobile, and personal self-hosting without introducing authentication.
