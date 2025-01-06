# Use the official ArchLinux image with a AUR helper installed
FROM ghcr.io/greyltc-org/archlinux-aur:paru

ARG REMOTE_USER
ARG REMOTE_UID
ARG REMOTE_GID

RUN pacman-key --init
RUN pacman -Syu --noconfirm

# Install required packages
RUN pacman -S base-devel git openssh nano protobuf rustup jdk21-openjdk gradle mkdocs-material --noconfirm

# Install gRPC UI
RUN aur-install grpcui-bin

RUN <<EOF
    groupadd -g ${REMOTE_GID} ${REMOTE_USER}
    useradd -m -u ${REMOTE_UID} -g ${REMOTE_GID} ${REMOTE_USER}
EOF

ENV HOME=/home/${REMOTE_USER}

USER ${REMOTE_USER}

# Install Rustup and set default toolchains
RUN rustup default nightly

# Install the wasm32-wasi target for nightly
RUN rustup target add wasm32-wasip1 --toolchain nightly

# Install binstall
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Add $HOME/.cargo/bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Install wasm-tools
RUN cargo binstall wasm-tools --no-confirm