# Notes

## HTML code coverage report

To get an HTML code coverage report, run these commands from the top level of the repository:

```bash
cargo install cargo-tarpaulin
cargo run tarpaulin -o Html
```

The report is saved to `tarpaulin-report.html`.

## Installing the GNU RISC-V compiler toolchain

The RISC-V gcc compiler is used to build the binaries that are used to test the RISC-V virtual machine in this repository.

To install it, follow the instructions [here](https://github.com/riscv-collab/riscv-gnu-toolchain). First, clone the repository and install prerequisites:

```bash
git clone https://github.com/riscv/riscv-gnu-toolchain
cd riscv-gnu-toolchain
sudo apt install autoconf automake autotools-dev curl python3 python3-pip libmpc-dev libmpfr-dev libgmp-dev gawk build-essential bison flex texinfo gperf libtool patchutils bc zlib1g-dev libexpat-dev ninja-build git cmake libglib2.0-dev
```

Next, build the compiler with newlib (the default C library) with debugging symbols including (the `-g` flag). Add `--enable-multilib` to build a compiler for both 32-bit and 64-bit RISC-V targets.

```bash
mkdir -p $HOME/opt/
./configure --prefix=$HOME/opt/riscv --enable-multilib CFLAGS=-g
make -j32
```

If you get errors like `error: RPC failed; curl 56 GnuTLS recv error (-54): Error in the pull function` while the `make` script is cloning repositories (like `gcc`, `binutils`, etc.), then wait and try again later (see  [here](https://github.com/riscv-collab/riscv-gnu-toolchain/issues/480)).

Make sure that `$HOME/opt/riscv/bin/` is in the path.

To uninstall the toolchain, run `rm -rf $HOME/opt/riscv`.
