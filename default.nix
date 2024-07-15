with import <nixpkgs> {};
mkShell{
    nativeBuildInputs = [rustc rust-analyzer cargo pkgsStatic.stdenv.cc gdb];
    RUSTC_BOOTSTRAP=1;
}
