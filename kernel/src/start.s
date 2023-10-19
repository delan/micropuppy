.extern LD_STACK_PTR // defined in linker.ld
.equ PSCI_SYSTEM_OFF, 0x84000008

.section ".text.startup"
.globl _start
_start:
    ldr x30, =LD_STACK_PTR
    mov sp, x30
    bl kernel_main

    ldr x0, =PSCI_SYSTEM_OFF
    hvc #0
