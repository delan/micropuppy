.extern INITIAL_SP // defined in linker.ld
.equ PSCI_SYSTEM_OFF, 0x84000008

.section ".text.startup"
.globl _start
_start:
    ldr x30, =INITIAL_SP
    mov sp, x30
    bl kernel_main

    ldr x0, =PSCI_SYSTEM_OFF
    hvc #0

.section ".text.vectors"

.globl vectors
vectors:
el0_synchronous_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
el0_irq_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
el0_fiq_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
el0_serror_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
elx_synchronous_vector:
    mov w0, #'+'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
elx_irq_vector:
    b elx_irq_wrapper
.align 7
elx_fiq_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
elx_serror_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower64_synchronous_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower64_irq_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower64_fiq_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower64_serror_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower32_synchronous_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower32_irq_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower32_fiq_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower32_serror_vector:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7

elx_irq_wrapper:
    sub sp, sp, #0x80
    str x0, [sp, #0x00]
    str x1, [sp, #0x04]
    str x2, [sp, #0x08]
    str x3, [sp, #0x0C]
    str x4, [sp, #0x10]
    str x5, [sp, #0x14]
    str x6, [sp, #0x18]
    str x7, [sp, #0x1C]
    str x8, [sp, #0x20]
    str x9, [sp, #0x24]
    str x10, [sp, #0x28]
    str x11, [sp, #0x2C]
    str x12, [sp, #0x30]
    str x13, [sp, #0x34]
    str x14, [sp, #0x38]
    str x15, [sp, #0x3C]
    str x16, [sp, #0x40]
    str x17, [sp, #0x44]
    str x18, [sp, #0x48]
    str x19, [sp, #0x4C]
    str x20, [sp, #0x50]
    str x21, [sp, #0x54]
    str x22, [sp, #0x58]
    str x23, [sp, #0x5C]
    str x24, [sp, #0x60]
    str x25, [sp, #0x64]
    str x26, [sp, #0x68]
    str x27, [sp, #0x6C]
    str x28, [sp, #0x70]
    str x29, [sp, #0x74]
    str x30, [sp, #0x78]
    str x31, [sp, #0x7C]
    bl elx_irq
    ldr x0, [sp, #0x00]
    ldr x1, [sp, #0x04]
    ldr x2, [sp, #0x08]
    ldr x3, [sp, #0x0C]
    ldr x4, [sp, #0x10]
    ldr x5, [sp, #0x14]
    ldr x6, [sp, #0x18]
    ldr x7, [sp, #0x1C]
    ldr x8, [sp, #0x20]
    ldr x9, [sp, #0x24]
    ldr x10, [sp, #0x28]
    ldr x11, [sp, #0x2C]
    ldr x12, [sp, #0x30]
    ldr x13, [sp, #0x34]
    ldr x14, [sp, #0x38]
    ldr x15, [sp, #0x3C]
    ldr x16, [sp, #0x40]
    ldr x17, [sp, #0x44]
    ldr x18, [sp, #0x48]
    ldr x19, [sp, #0x4C]
    ldr x20, [sp, #0x50]
    ldr x21, [sp, #0x54]
    ldr x22, [sp, #0x58]
    ldr x23, [sp, #0x5C]
    ldr x24, [sp, #0x60]
    ldr x25, [sp, #0x64]
    ldr x26, [sp, #0x68]
    ldr x27, [sp, #0x6C]
    ldr x28, [sp, #0x70]
    ldr x29, [sp, #0x74]
    ldr x30, [sp, #0x78]
    ldr x31, [sp, #0x7C]
    add sp, sp, #0x80
    eret
