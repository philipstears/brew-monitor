# Build Instructions

## Nix/NixOS

## Windows

## macOS

## Ubuntu

### Pre-requisites

#### Install packages from apt

```
sudo apt install build-essential npm libsqlite3-dev
```

- `build-essential` is required to make the GNU compiler collection available
- `npm` is required to build the front-end
- `libsqlite3-dev` is required for access to the brew-monitor database


#### Install rustup

Follow the instructions at https://rustup.rs/ to install rustup for managing your rust installation.

#### Install docker (optional, for cross compilation)

Follow the instructions at https://docs.docker.com/engine/install/ubuntu/

#### Install cross (optional, for cross compilation)

Cross is used for building for the Raspberry Pi.

```
cargo install cross
```

#### Install cargo-deb (optional, for packaging)

cargo-deb is used for building debian packages for the Raspberry Pi.

