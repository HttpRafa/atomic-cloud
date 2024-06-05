# Use the official ArchLinux image with a AUR helper installed
FROM ghcr.io/greyltc-org/archlinux-aur:paru

RUN pacman-key --init
RUN pacman -Syu --noconfirm && pacman -S git openssh nano rustup protobuf --noconfirm

# Install Rustup and set default toolchains
RUN rustup default stable
RUN rustup toolchain install nightly

# Install the wasm32-wasi target for nightly
RUN rustup target add wasm32-wasip1 --toolchain nightly

# Add $HOME/.cargo/bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Install wasm-tools
RUN cargo install wasm-tools

# Install gRPC UI
RUN aur-install grpcui-bin