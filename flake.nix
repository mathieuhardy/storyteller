{
  description = "Rust nightly dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ rust-overlay.overlays.default ];
      };

      # The built SvelteKit SPA (`frontend/build/`), packaged separately so
      # `rustPlatform.buildRustPackage` below can drop it in place before
      # `cargo build` — `#[derive(Embed)]` reads that folder at *compile*
      # time (storyteller-server/src/frontend.rs), so it must exist first.
      #
      # `npmDepsHash` is a placeholder: Nix needs the real fixed-output hash
      # of `frontend/package-lock.json`'s dependency tree, which can only be
      # computed by actually fetching it. Run `nix build .#frontend`; it will
      # fail with the correct hash to paste in here (standard Nix workflow —
      # this can't be produced without network access to npm's registry).
      frontend = pkgs.buildNpmPackage {
        pname = "storyteller-frontend";
        version = "0.1.0";
        src = ./frontend;
        npmDepsHash = pkgs.lib.fakeHash;
        installPhase = ''
          mkdir -p $out
          cp -r build/. $out/
        '';
      };

      storyteller-server = pkgs.rustPlatform.buildRustPackage {
        pname = "storyteller-server";
        version = "0.1.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
        nativeBuildInputs = [ pkgs.pkg-config ];
        cargoBuildFlags = [
          "-p"
          "storyteller-server"
        ];
        # Same reason the Docker build stages the frontend before `cargo
        # build`: the embed has to see real files at compile time.
        postPatch = ''
          mkdir -p frontend/build
          cp -r ${frontend}/. frontend/build/
        '';
        # The workspace's own test suite runs `cargo test` directly (CI, or a
        # contributor's machine) against whatever `frontend/build/` happens
        # to exist there — usually nothing, which is exactly the "no
        # frontend embedded" case those tests assert on (see
        # `routes_are_versioned` in storyteller-server/tests/api.rs). Here a
        # *real* frontend is always staged first, which would make that
        # specific assertion fail for a reason that has nothing to do with
        # whether the package itself is correct — so the package build
        # doesn't re-run that suite.
        doCheck = false;
        meta.mainProgram = "storyteller-server";
      };
    in
    {
      devShells.x86_64-linux.default = pkgs.mkShell {
        buildInputs = [
          (pkgs.rust-bin.nightly.latest.default)
          pkgs.pkg-config
          pkgs.fuse3
        ];
      };

      packages.x86_64-linux = {
        default = storyteller-server;
        inherit storyteller-server frontend;
      };

      apps.x86_64-linux.default = {
        type = "app";
        program = "${storyteller-server}/bin/storyteller-server";
      };
    };
}
