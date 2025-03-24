FROM debian:unstable

ARG REMOTE_USER
ARG REMOTE_UID
ARG REMOTE_GID

# Update and upgrade the system
RUN apt update && apt upgrade -y

# Install necessary packages
RUN apt install -y \
    build-essential git gh \
    mkdocs-material \
    protobuf-compiler \
    rustup \
    openjdk-21-jdk gradle

RUN <<EOF
    groupadd -g ${REMOTE_GID} ${REMOTE_USER}
    useradd -m -u ${REMOTE_UID} -g ${REMOTE_GID} ${REMOTE_USER}
EOF

ENV HOME=/home/${REMOTE_USER}

# Switch to bash as default shell
RUN chsh -s /bin/bash ${REMOTE_USER}

USER ${REMOTE_USER}

# Switch to nano as default editor
ENV EDITOR=nano

# Install Rustup and set default toolchains
RUN rustup default nightly