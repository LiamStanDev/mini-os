# -----------------------------------------------------------------------------
# switch.S - Task context switch routines for RISC-V
#
# This file implements the low-level context switch routine (__switch) for RISC-V.
# It saves the current task's context (registers) to memory and restores the next
# task's context from memory, enabling preemptive multitasking.
#
# - __switch: Saves the current task's context (ra, sp, s0-s11) to the current
#   task's TaskContext structure, then loads the next task's context from its
#   TaskContext structure and returns to the next task.
#
# TaskContext layout in memory (offsets in 8-byte words):
#   0:  ra (return address)
#   1:  sp (stack pointer)
#   2-13: s0-s11 (callee-saved registers)
# -----------------------------------------------------------------------------

.altmacro
.macro SAVE_SN n
  sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
  ld s\n, (\n+2)*8(a1)
.endm

  .section .text
  .globl __switch
__switch:
  # __switch(
  #   current_task_ctx_ptr: *mut TaskContext,
  #   next_task_ctx_ptr: *const TaskContext
  # )
  
  # save current task
  sd ra, 0*8(a0)
  sd sp, 1*8(a0)
  .set n, 0
  .rept 12
    SAVE_SN %n
    .set n, n+1
  .endr

 # load next task
 ld ra, 0*8(a1)
 ld sp, 1*8(a1)
 .set n, 0
 .rept 12
   LOAD_SN %n
   .set n, n+1
 .endr
 ret
