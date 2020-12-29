FROM ubuntu:latest

# Make sure that the user and groups in the container
# match those outside so the user creates files with the
# right permissions, and can access the host's docker
ARG BUILD_USER_UID
ARG BUILD_USER_GID
ARG DOCKER_GID

RUN groupadd --gid $DOCKER_GID docker && \
    apt-get update && \
    apt-get install -y sudo curl docker.io build-essential && \
    rm -rf /var/lib/apt/lists/*

# Make sure that files we create are owned by the user, not by root
RUN groupadd --gid $BUILD_USER_GID build-group && \
    useradd  --uid $BUILD_USER_UID --gid $BUILD_USER_GID --groups docker,sudo --create-home build-user

USER build-user

# Make bash use a login shell so that it imports bashrc
SHELL [ "/bin/bash", "-l", "-c" ]

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable -y

RUN rustup component add rust-src

# Add cross compilation targets
RUN rustup target add armv7-unknown-linux-gnueabihf

RUN cargo install cross && cargo install cargo-deb

# Make sure cross knows that it is running in a container
ENV CROSS_DOCKER_IN_DOCKER=true
