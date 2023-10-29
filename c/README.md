# Installing the RISC-V C newlib toolchain


```bash
# Download the toolchain
git clone https://github.com/riscv/riscv-gnu-toolchain

# Configure the compiler to use newlib, target the 32-bit base integer
# (I) instruction set, and use the ILP32 ABI.
./configure --prefix=$HOME/opt/riscv --with-arch=rv32i --with-abi=ilp32

# This will also install the compiler
make -j32
```
