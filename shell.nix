let
  mozilla =
    builtins.fetchGit {
      name = "nixpkgs-mozilla";
      url = https://github.com/mozilla/nixpkgs-mozilla/;
    };

  nixPackages =
    import <nixpkgs> {
      overlays = [
        (import mozilla)
      ];
    };

  # NOTE: this is the last version before the layout of the source
  #  code changed, pinning to this version for now, because the
  # version of rustracer in nixpkgs doesn't have the patch to support
  # the new layout
  rustChannel = nixPackages.rustChannelOf { date = "2020-08-27"; channel = "nightly"; };

  rust =
    (rustChannel.rust.override {
      extensions = [
        "rust-src"
        "rls-preview"
        "rust-analysis"
        "rustfmt-preview"
        "clippy-preview"
      ];
    });

in

  with nixPackages;

  mkShell {
    buildInputs = with pkgs; [
      rust
      rustracer
      nodejs
      (sqlite.override {
        inherit readline ncurses;
        interactive = true;
      })
    ];

    RUST_SRC_PATH = "${rustChannel.rust-src}/lib/rustlib/src/rust/src";
    RACER_PATH = "${rustracer}/bin/racer";
  }
