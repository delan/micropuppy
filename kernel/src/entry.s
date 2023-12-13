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

.macro task_save
    // save task general-purpose registers
    sub sp, sp, #0x120
    stp x0, x1, [sp, #0x00]
    stp x2, x3, [sp, #0x10]
    stp x4, x5, [sp, #0x20]
    stp x6, x7, [sp, #0x30]
    stp x8, x9, [sp, #0x40]
    stp x10, x11, [sp, #0x50]
    stp x12, x13, [sp, #0x60]
    stp x14, x15, [sp, #0x70]
    stp x16, x17, [sp, #0x80]
    stp x18, x19, [sp, #0x90]
    stp x20, x21, [sp, #0xa0]
    stp x22, x23, [sp, #0xb0]
    stp x24, x25, [sp, #0xc0]
    stp x26, x27, [sp, #0xd0]
    stp x28, x29, [sp, #0xe0]
    stp x30, x31, [sp, #0xf0]

    // save task PC and SP
    mrs x0, ELR_EL1
    mrs x1, SP_EL0
    stp x0, x1, [sp, #0x100]

    // save task PSTATE (SPSR - saved PSR)
    mrs x0, SPSR_EL1
    str x0, [sp, #0x110]
.endm

.macro task_restore
    // restore task PSTATE
    ldr x0, [sp, #0x110]
    msr SPSR_EL1, x0

    // restore task PC and SP
    ldp x0, x1, [sp, #0x100]
    msr SP_EL0, x1
    msr ELR_EL1, x0

    // restore task general-purpose registers
    ldp x30, x31, [sp, #0xf0]
    ldp x28, x29, [sp, #0xe0]
    ldp x26, x27, [sp, #0xd0]
    ldp x24, x25, [sp, #0xc0]
    ldp x22, x23, [sp, #0xb0]
    ldp x20, x21, [sp, #0xa0]
    ldp x18, x19, [sp, #0x90]
    ldp x16, x17, [sp, #0x80]
    ldp x14, x15, [sp, #0x70]
    ldp x12, x13, [sp, #0x60]
    ldp x10, x11, [sp, #0x50]
    ldp x8, x9, [sp, #0x40]
    ldp x6, x7, [sp, #0x30]
    ldp x4, x5, [sp, #0x20]
    ldp x2, x3, [sp, #0x10]
    ldp x0, x1, [sp, #0x00]

    add sp, sp, #0x120
.endm

el0_synchronous_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'A'
    strb w0, [x1]
    eret

.align 7
el0_irq_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'B'
    strb w0, [x1]
    eret

.align 7
el0_fiq_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'C'
    strb w0, [x1]
    eret

.align 7
el0_serror_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'D'
    strb w0, [x1]
    eret

.align 7
elx_synchronous_vector:
    mov x1, #0x9000000
    mov w0, #'\n'
    strb w0, [x1]
    mov w0, #'E'
    strb w0, [x1]
    mov w0, #'L'
    strb w0, [x1]
    mov w0, #'x'
    strb w0, [x1]
    mov w0, #'S'
    strb w0, [x1]
    mov w0, #'E'
    strb w0, [x1]
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'\n'
    strb w0, [x1]
    b .
    eret

.align 7
elx_irq_vector:
    b elx_irq_wrapper

.align 7
elx_fiq_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'G'
    strb w0, [x1]
    eret

.align 7
elx_serror_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'H'
    strb w0, [x1]
    eret

.align 7
lower64_synchronous_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'I'
    strb w0, [x1]
    eret

.align 7
lower64_irq_vector:
    b lower64_irq_wrapper

.align 7
lower64_fiq_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'K'
    strb w0, [x1]
    eret

.align 7
lower64_serror_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'L'
    strb w0, [x1]
    eret

.align 7
lower32_synchronous_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'M'
    strb w0, [x1]
    eret

.align 7
lower32_irq_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'N'
    strb w0, [x1]
    eret

.align 7
lower32_fiq_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'O'
    strb w0, [x1]
    eret

.align 7
lower32_serror_vector:
    mov x1, #0x9000000
    mov w0, #'!'
    strb w0, [x1]
    mov w0, #'P'
    strb w0, [x1]
    eret

.align 7
elx_irq_wrapper:
    task_save
    mov x0, sp

    bl elx_irq

    mov sp, x0
    task_restore
    eret

lower64_irq_wrapper:
    b elx_irq_wrapper


.global scheduler_start
scheduler_start:
    mov sp, x0
    task_restore
    eret
