### Installing the GNU RISC-V compiler toolchain

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

### Installing the `kcc` C-semantics checker

Install from source using the instruction [here](https://github.com/kframework/c-semantics/blob/master/INSTALL.md)

```bash
# Modified to clang libclang-dev and 
sudo apt-get install --yes \
  maven git openjdk-8-jdk flex libgmp-dev libffi-dev \
  libmpfr-dev build-essential cmake zlib1g-dev \
  diffutils libuuid-tiny-perl \
  libstring-escape-perl libstring-shellquote-perl \
  libgetopt-declare-perl opam pkg-config \
  libapp-fatpacker-perl liblocal-lib-perl \
  clang libclang-dev

git clone https://github.com/kframework/c-semantics.git
cd c-semantics
git submodule update --init --recursive

opam init # This seemed necessary. Choose y to modify .profile

eval $(opam config env)
eval $(perl -I "~/perl5/lib/perl5" -Mlocal::lib)
make -j4 --output-sync=line
```












; for example:


```bash
# Install prerequisites
sudo apt-get install python3 gcc clang-12 libjemalloc-dev libgmp-dev libmpfr-dev libfmt-dev z3

cd /usr/local
sudo wget https://github.com/runtimeverification/match/releases/download/snapshot-b9c152d/rv-match.tar.gz

# The next line will extract directly into /usr/local/{lib,bin}
sudo tar xvf rv-match.tar.gz

# Delete the old tar file
sudo rm rv-match.tar.gz
```

Now you should be able to run `kcc`
