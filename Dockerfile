# Self-host image (docs/architecture.md §5, M6): storyteller-server + the
# built SvelteKit front, embedded into the binary at compile time
# (storyteller-server/src/frontend.rs) — one reproducible artifact, no
# separate static-files volume to keep in sync with the binary.
#
#   docker build -t storyteller .
#   docker run -p 8787:8787 -v /path/to/your/novel:/data storyteller

# --- frontend: build the static SPA -----------------------------------------
FROM node:22-bookworm-slim AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# --- server: build the Rust binary, with the frontend embedded -------------
# `rust-version` in Cargo.toml (1.82) is the workspace's MSRV floor, not a
# ceiling: some transitive dependency (as pinned by Cargo.lock) needs a newer
# edition than 1.82's cargo supports, so the build image tracks current
# stable rather than that floor.
FROM rust:1-bookworm AS server
# rusqlite's `bundled` feature compiles SQLite from source (storyteller-core
# needs a C compiler for that; the plain `rust` image, unlike `-slim`, already
# ships build-essential, but this is explicit rather than relying on it).
RUN apt-get update && apt-get install -y --no-install-recommends build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY storyteller-core/Cargo.toml storyteller-core/Cargo.toml
COPY storyteller-server/Cargo.toml storyteller-server/Cargo.toml
COPY storyteller-core storyteller-core
COPY storyteller-server storyteller-server
# The frontend must be in place *before* `cargo build`: `#[derive(Embed)]`
# reads `frontend/build/` at compile time (ADR — see frontend.rs).
COPY --from=frontend /app/frontend/build frontend/build
RUN cargo build --release -p storyteller-server

# --- runtime: just the binary ------------------------------------------------
FROM debian:bookworm-slim AS runtime
COPY --from=server /app/target/release/storyteller-server /usr/local/bin/storyteller-server

# Local-first, no auth (ADR 0003): the binary itself defaults to loopback-only.
# Self-hosting is an explicit choice the operator makes by publishing the
# port — binding every interface *inside* the container is what that choice
# looks like; it is still only reachable via whatever `docker run -p` exposes.
ENV STORYTELLER_BIND=0.0.0.0:8787
ENV STORYTELLER_PROJECT=/data
EXPOSE 8787
VOLUME /data

# Runs as root: `/data` is an arbitrary host-bind-mounted project folder
# (docs/adr/0004-one-folder-per-project.md), owned by whatever uid the
# operator happens to have on the host — a fixed non-root image uid would
# only be able to write it by coincidence. Same trust boundary as running the
# binary directly on the host (ADR 0003: single operator, no auth); the
# container adds packaging, not a security boundary between users.
ENTRYPOINT ["storyteller-server"]
