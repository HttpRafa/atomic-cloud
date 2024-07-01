# Use the official ArchLinux image with a AUR helper installed
FROM ghcr.io/greyltc-org/archlinux-aur:paru

RUN pacman-key --init
RUN pacman -Syu --noconfirm && pacman -S git openssh nano rustup protobuf --noconfirm

# Install Rustup and set default toolchains
RUN rustup default stable
RUN rustup toolchain install nightly

# Install the wasm32-wasi target for nightly
RUN rustup target add wasm32-wasip1 --toolchain nightly

# Install binstall
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Add $HOME/.cargo/bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Install wasm-tools
RUN cargo binstall wasm-tools --no-confirm

# Install gRPC UI
RUN aur-install grpcui-bin