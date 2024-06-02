{pkgs ? import <nixpkgs> {config.allowUnfree = true;}}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    # Install jetbrains ides
    jetbrains.idea-ultimate

    # Rust
    rustup

    # Java 21 and Gradle
    temurin-bin
    gradle

    # Protobuf
    protobuf_26
    grpcui

    # Wasm tooling
    wasm-tools
    wabt

    # Github
    act
  ];
}
