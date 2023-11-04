.section ".text.vectors"

.globl vectors
vectors:
el0_synchronous:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
el0_irq:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
el0_fiq:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
el0_serror:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
elx_synchronous:
    mov w0, #'+'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
elx_irq:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
elx_fiq:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
elx_serror:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower64_synchronous:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower64_irq:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower64_fiq:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower64_serror:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower32_synchronous:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower32_irq:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower32_fiq:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
.align 7
lower32_serror:
    mov w0, #'.'
    mov x1, #0x9000000
    strb w0, [x1]
    eret
