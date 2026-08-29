{
  description = "Storyteller — personal knowledge base";

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
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      # The built SvelteKit SPA (`frontend/build/`), packaged separately so
      # `rustPlatform.buildRustPackage` below can drop it in place before
      # `cargo build` — `#[derive(Embed)]` reads that folder at *compile*
      # time (storyteller-server/src/frontend.rs), so it must exist first.
      #
      # If `npmDepsHash` becomes stale after updating frontend dependencies,
      # run `nix build .#frontend` — it will fail with the new hash to paste here.
      frontend = pkgs.buildNpmPackage {
        pname = "storyteller-frontend";
        version = "0.1.0";
        src = ./frontend;
        npmDepsHash = "sha256-fFd4006YltAzOd90u9jQ4j5FSKwwJKnBQY/vuZSVBq4=";
        installPhase = ''
          mkdir -p $out
          cp -r build/. $out/
        '';
      };

      storyteller-tauri = pkgs.rustPlatform.buildRustPackage {
        pname = "storyteller";
        version = "0.1.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = with pkgs; [
          pkg-config
          wrapGAppsHook3
          copyDesktopItems
        ];

        buildInputs = with pkgs; [
          webkitgtk_4_1
          gtk3
          libayatana-appindicator
          librsvg
          openssl
        ];

        cargoBuildFlags = [
          "-p"
          "storyteller-tauri"
        ];

        # Stage the frontend before cargo build (rust-embed reads at compile time)
        postPatch = ''
          mkdir -p frontend/build
          cp -r ${frontend}/. frontend/build/
        '';

        postInstall = ''
          # Install icon
          mkdir -p $out/share/icons/hicolor/128x128/apps
          cp storyteller-tauri/icons/128x128.png $out/share/icons/hicolor/128x128/apps/storyteller.png

          # Rename binary for cleaner desktop integration
          mv $out/bin/storyteller-tauri $out/bin/storyteller
        '';

        desktopItems = [
          (pkgs.makeDesktopItem {
            name = "storyteller";
            exec = "storyteller";
            icon = "storyteller";
            desktopName = "Storyteller";
            comment = "Personal knowledge base";
            categories = [ "Office" "Utility" ];
            terminal = false;
          })
        ];

        doCheck = false;
        meta.mainProgram = "storyteller";
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
        postPatch = ''
          mkdir -p frontend/build
          cp -r ${frontend}/. frontend/build/
        '';
        doCheck = false;
        meta.mainProgram = "storyteller-server";
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          (pkgs.rust-bin.nightly.latest.default)
          pkgs.pkg-config
          pkgs.fuse3
          # Tauri dev dependencies
          pkgs.webkitgtk_4_1
          pkgs.gtk3
          pkgs.libayatana-appindicator
          pkgs.librsvg
        ];
      };

      packages.${system} = {
        default = storyteller-tauri;
        inherit storyteller-tauri storyteller-server frontend;
      };

      apps.${system}.default = {
        type = "app";
        program = "${storyteller-tauri}/bin/storyteller";
      };
    };
}
