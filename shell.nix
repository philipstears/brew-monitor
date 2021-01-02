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

  rustChannel = nixPackages.latest.rustChannels.stable;

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
      sqlite
    ];

    RUST_SRC_PATH = "${rustChannel.rust-src}/lib/rustlib/src/rust/src";
    RACER_PATH = "${rustracer}/bin/racer";
  }
