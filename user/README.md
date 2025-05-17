### Test Binaries by qemu-riscv64

#### Method 01
Because qemu-riscv64 is a semi-emulator for riscv64 linux, so the binaries need to compile with linux system call format (ABI). First we need
to comment out .cargo/config.toml for our own os linker setup.
```toml
# user/.cargo/config.toml
[build]
target = "riscv64gc-unknown-none-elf"

#[target.riscv64gc-unknown-none-elf]
#rustflags = [
#    "-Clink-args=-Tsrc/linker.ld", 
#    "-Cforce-frame-pointers=yes"
#]
```

and then comment out the clear_bss related code
```rust
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    // clear_bss();
    exit(main());
    panic!("unreachable after sys_exist!");
}
```

you can run
```sh
qemu-riscv64 target/riscv64gc-unknown-none-elf/release/00hello_world
```
```
```

#### Method 02

linker.ld need to add align(4K) ref: 

```config
OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x80400000;

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }


    . = ALIGN(4K);
    .bss : {
        start_bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        end_bss = .;
    }
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}
```
