# -----------------------------------------------------------------------------
# trap.S - Trap entry and context switch routines for RISC-V
#
# This file implements the low-level trap entry (__alltraps) and context restore
# (__restore) routines for RISC-V, handling the saving and restoring of all
# general-purpose registers and control/status registers required for trap
# handling and context switching between user and kernel mode.
#
# - __alltraps: Entry point for all traps (interrupts, exceptions, syscalls).
#   Saves user context into TrapContext, switches to kernel address space,
#   and jumps to the kernel trap handler.
#
# - __restore: Restores user context from TrapContext and returns to user mode.
#
# TrapContext layout in memory (offsets in 8-byte words):
#   0-31:  x[0]..x[31] (general-purpose registers, x[2]=sp is handled specially)
#   32:    sstatus
#   33:    sepc
#   34:    kernel_satp
#   35:    kernel_sp
#   36:    trap_handler
# -----------------------------------------------------------------------------

#.altmacro
#.macro SAVE_GP n
#  sd x\n, \n*8(sp)
#.endm
#.macro LOAD_GP n
#    ld x\n, \n*8(sp)
#.endm


#  .section .text.trampoline
#  .globl __alltraps
#  .globl __restore
#  .align 2 # riscv specification
#__alltraps:
#  # Save user stack pointer and swap with sscratch (kernel/user stack exchange)
#  # sp -> user stack, sscratch -> *TrapContext in user stack
#  csrrw sp, sscratch, sp
#  # sp -> *TrapContext in user space, sscratch -> user stack
#
#  # Save general-purpose registers except sp(x2)/tp(x4)
#  sd x1, 1*8(sp) # save x1(ra)
#  # skip x2(sp)
#  sd x3, 3*8(sp) # save x3(gp)
#  # Save x5-x31 using macro
#  .set n, 5
#  .rept 27
#    SAVE_GP %n
#    .set n, n+1
#  .endr
#  
#  # Save control/status registers to TrapContext
#  csrr t0, sstatus
#  csrr t1, sepc
#  csrr t2, sscratch  # t2 -> user stack
#  sd t0, 32*8(sp)    # TrapContext.sstatus
#  sd t1, 33*8(sp)    # TrapContext.sepc
#  sd t2, 2*8(sp)     # TrapContext.x[2]
#
#  # Load kernel_satp, trap_handler, and kernel_sp from TrapContext
#  ld t0, 34*8(sp)    # kernel_satp  
#  ld sp, 35*8(sp)    # kernel_sp    
#  ld t1, 36*8(sp)    # trap_handler    
#
#  # Switch to kernel address space and flush TLB
#  csrw satp, t0
#  sfence.vma
#
#  # Jump to kernel trap handler
#  jr t1

#__restore:
#  # a0: pointer to TrapContext, a1: user satp
#  csrw satp, a1
#  sfence.vma # flush TLB
#
#  csrw sscratch, a0
#  mv sp, a0
#
#  # Restore control/status registers
#  ld t0, 32*8(sp)    # TrapContext.sstatus
#  ld t1, 33*8(sp)    # TrapContext.sepc
#  csrw sstatus, t0
#  csrw sepc, t1
#
#  # Restore general-purpose registers except sp(x2)/tp(x4)
#  ld x1, 1*8(sp)
#  ld x3, 3*8(sp)
#  .set n, 5
#  .rept 27
#    LOAD_GP %n
#    .set n, n+1
#  .endr
#
#  ld sp, 2*8(sp)   # Restore user sp     
#  sret             # Return to user mode 



.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # load kernel_satp into t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp
    ld sp, 35*8(sp)
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret
