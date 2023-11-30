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

Next, build the compiler for the integer instruction set with multiplication, division and atomics (no floating point), and without the compressed instructions. Use the `lp64` ABI:

```bash
mkdir -p $HOME/opt/
./configure --prefix=$HOME/opt/ --with-arch=rv64imda --with-abi=lp64
make -j32 linux
```

Make sure that `$HOME/opt/bin/` is in the path. The C compiler is called `riscv64-unknown-linux-gnu-gcc`.
