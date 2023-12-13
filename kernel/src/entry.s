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
.macro define_vector_trampoline, addr:req, source:req, type:req
    .org \addr
    vector_\source\()_\type\()_trampoline:
        b vector_\source\()_\type\()_wrapper
.endm

define_vector_trampoline 0x000, el0, synchronous
define_vector_trampoline 0x080, el0, irq
define_vector_trampoline 0x100, el0, fiq
define_vector_trampoline 0x180, el0, serror

define_vector_trampoline 0x200, elx, synchronous
define_vector_trampoline 0x280, elx, irq
define_vector_trampoline 0x300, elx, fiq
define_vector_trampoline 0x380, elx, serror

define_vector_trampoline 0x400, lower64, synchronous
define_vector_trampoline 0x480, lower64, irq
define_vector_trampoline 0x500, lower64, fiq
define_vector_trampoline 0x580, lower64, serror

define_vector_trampoline 0x600, lower32, synchronous
define_vector_trampoline 0x680, lower32, irq
define_vector_trampoline 0x700, lower32, fiq
define_vector_trampoline 0x780, lower32, serror

// **These macros MUST be kept in sync with the `Context` struct defined in `task.rs`.**
.macro task_save
    // GPRs => context.gprs
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

    // PC => context.pc
    // SP => context.sp
    mrs x0, ELR_EL1
    mrs x1, SP_EL0
    stp x0, x1, [sp, #0x100]

    // PSTATE => context.psr
    mrs x0, SPSR_EL1
    str x0, [sp, #0x110]
.endm

.macro task_restore
    // context.psr => PSTATE
    ldr x0, [sp, #0x110]
    msr SPSR_EL1, x0

    // context.sp => SP
    // context.pc => PC
    ldp x0, x1, [sp, #0x100]
    msr SP_EL0, x1
    msr ELR_EL1, x0

    // context.gprs => GPRs
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

.macro define_vector_stub, source:req, type:req
    vector_\source\()_\type\()_wrapper:
        eret
.endm

.macro define_vector_task, source:req, type:req
    vector_\source\()_\type\()_wrapper:
        task_save
        mov x0, sp

        bl vector_\source\()_\type

        mov sp, x0
        task_restore
        eret
.endm

define_vector_stub el0, synchronous
define_vector_stub el0, irq
define_vector_stub el0, fiq
define_vector_stub el0, serror

define_vector_stub elx, synchronous
define_vector_stub elx, irq
define_vector_stub elx, fiq
define_vector_stub elx, serror

define_vector_task lower64, synchronous
define_vector_task lower64, irq
define_vector_task lower64, fiq
define_vector_task lower64, serror

define_vector_stub lower32, synchronous
define_vector_stub lower32, irq
define_vector_stub lower32, fiq
define_vector_stub lower32, serror

.global task_start
task_start:
    mov sp, x0
    task_restore
    eret
