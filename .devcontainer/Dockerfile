# Use the official ArchLinux image with a AUR helper installed
FROM ghcr.io/greyltc-org/archlinux-aur:paru

ARG REMOTE_USER
ARG REMOTE_UID
ARG REMOTE_GID

RUN pacman-key --init
RUN pacman -Syu --noconfirm

# Install required packages
RUN pacman -S base-devel git less openssh nano protobuf rustup jdk21-openjdk gradle mkdocs-material --noconfirm

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