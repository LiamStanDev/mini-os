.altmacro
.macro SAVE_GP n
  sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm


  .section .text
  .globl __alltraps
  .align 2 # riscv specification
__alltraps:
  # csrrw rd, csr, rs: means to save csr to rd then write rs to csr
  # sp -> UserStack, sscratch -> KernelStack
  csrrw sp, sscratch, sp
  # sp -> KernelStack, sscratch -> UserStack

  # allocate 'TrapContext' on Kernel stack
  addi sp, sp, -34*8

  # save general-purpuse registers except sp(x2)/tp(x4)
  sd x1, 1*8(sp) # save x1(ra)
  # skip x2(sp)
  sd x3, 3*8(sp) # save x3(gp)
  # save x5-x31
  .set n, 5
  .rept 27
    SAVE_GP %n
    .set n, n+1
  .endr
  

  # save csr: csr need to use other registers
  csrr t0, sstatus
  csrr t1, sepc
  csrr t2, sscratch # t2 -> UserStack
  sd t0, 32*8(sp) # TrapContext.sstatus
  sd t1, 33*8(sp) # TrapContext.sepc
  sd t2, 2*8(sp) # TrapContext.x[2]

  # input argument of trap_handler(cx: &mut TrapContext)
  mv a0, sp
  call trap_handler

  .section .text
  .globl __restore
  .align 2 # riscv specification
__restore:
  # sp -> kernel stack, sscratch -> user stack
  # restore csr
  ld t0, 32*8(sp) # TrapContext.sstatus
  ld t1, 33*8(sp) # TrapContext.sepc
  ld t2, 2*8(sp) # TrapContex.x[2] 
  csrw sstatus, t0
  csrw sepc, t1
  csrw sscratch, t2

  # restore general-purpuse registers except sp(x2)/tp(x4)
  ld x1, 1*8(sp)
  ld x3, 3*8(sp)
  .set n, 5
  .rept 27
    LOAD_GP %n
    .set n, n+1
  .endr

  # pop 'TrapContext'
  addi sp, sp, 34*8
  csrrw sp, sscratch, sp
  # sp -> user stack, sscratch -> kernel stack
  sret
