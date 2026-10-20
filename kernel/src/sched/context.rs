//! Low-level thread context switching for x86_64

use core::arch::global_asm;

global_asm!(r#"
.global context_switch
/*
    rdi = prev_rsp_ptr: *mut u64
    rsi = next_rsp: u64
    rdx = next_cr3: u64
*/
context_switch:
    /* Save callee-preserved registers */
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15
    pushfq

    /* Save current stack pointer to *rdi */
    mov [rdi], rsp

    /* Switch address space if next_cr3 != 0 */
    test rdx, rdx
    jz 1f
    mov rax, cr3
    cmp rax, rdx
    je 1f
    mov cr3, rdx
1:
    /* Load new stack pointer */
    mov rsp, rsi

    /* Restore registers from new stack */
    popfq
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp

    ret
"#);

extern "C" {
    pub fn context_switch(prev_rsp: *mut u64, next_rsp: u64, next_cr3: u64);
}
