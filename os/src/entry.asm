 # NOTE: the code is grow from low address to high address

  .section .text.entry
  .globl _start
_start:
  la sp, boot_stack_top
  call rust_main

  .section .bss.stack
  .globl boot_stack_lower_bound
boot_stack_lower_bound:
  .space 4096 * 16 # 4KBi * 16 = 64KiB
  .globl boot_stack_top
boot_stack_top:
