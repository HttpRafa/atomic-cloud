# Credit: https://ziap.github.io/blog/nixos-cross-compilation/
{
  description = "Rust flake for Atomic Cloud";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;config.allowUnfree = true;};
  in {
    devShell.${system} = let
      targetName = {
        mingw = "x86_64-w64-mingw32";
        musl = "x86_64-unknown-linux-musl";
      };

      # Generate the cross compilation packages import
      pkgsCross = builtins.mapAttrs (name: value:
        import pkgs.path {
          system = system;
          crossSystem = {
            config = value;
          };
        })
      targetName;

      # Grab the corresponding C compiler binaries
      ccPkgs = builtins.mapAttrs (name: value: value.stdenv.cc) pkgsCross;
      cc = builtins.mapAttrs (name: value: "${value}/bin/${targetName.${name}}-cc") ccPkgs;
    in
      pkgs.mkShell {
        buildInputs =
          with pkgs; [
            # Rust/Controller
            rustup
            protobuf

            # Java
            gradle
            temurin-bin

            # Go/Docs
            go
            hugo

            # IDEs
            jetbrains.idea-ultimate
          ]
          ++ builtins.attrValues ccPkgs;

        # Set the default target to the first available target
        CARGO_BUILD_TARGET = let
          toolchainStr = builtins.readFile ./rust-toolchain.toml;
          targets = (builtins.fromTOML toolchainStr).toolchain.targets;
        in
          builtins.head targets;

        # Set up the C compiler
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = cc.musl;
        CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = cc.mingw;

        # Set up the C linker
        CC_x86_64_unknown_linux_musl = cc.musl;
        CC_x86_64_pc_windows_gnu = cc.mingw;

        RUSTFLAGS = builtins.map (a: "-L ${a}/lib") [
          pkgsCross.mingw.windows.mingw_w64_pthreads
        ];

        shellHook = ''
          # Install and display the current toolchain
          rustup show
        '';
      };
  };
}
