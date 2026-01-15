use std::arch::global_asm;

/// Macro principale pour syscalls indirects x86_64
#[macro_export]
macro_rules! syscall {
    ($function_name:expr, $($y:expr), +) => {{
        let (ssn, addr) = $crate::syscalls::resolve::get_ssn($crate::obf!($function_name));
        let mut cnt: u32 = 0;
        $(
            let _ = $y;
            cnt += 1;
        )+
        $crate::syscalls::syscall::do_syscall(ssn, addr, cnt, $($y), +)
    }}
}

// Code assembleur x64 pour indirect syscall
//
// Fonctionnement:
// 1. Sauvegarder rsi, rdi, r12 (registres non-volatiles)
// 2. Charger SSN dans eax
// 3. Charger l'adresse du gadget "syscall; ret" dans r12
// 4. Copier les arguments selon la calling convention Windows x64
// 5. Restaurer les registres
// 6. Jump vers le gadget dans ntdll.dll (indirect syscall)
#[cfg(target_arch = "x86_64")]
global_asm!(
    "
.global do_syscall

.section .text

do_syscall:
    mov [rsp - 0x8],  rsi
    mov [rsp - 0x10], rdi
    mov [rsp - 0x18], r12

    mov eax, ecx
    mov r12, rdx
    mov rcx, r8

    mov r10, r9
    mov  rdx,  [rsp + 0x28]
    mov  r8,   [rsp + 0x30]
    mov  r9,   [rsp + 0x38]

    sub rcx, 0x4
    jle skip

    lea rsi,  [rsp + 0x40]
    lea rdi,  [rsp + 0x28]

    rep movsq
skip:

    mov rcx, r12

    mov rsi, [rsp - 0x8]
    mov rdi, [rsp - 0x10]
    mov r12, [rsp - 0x18]

    jmp rcx

"
);

// Signature de la fonction assembleur
#[cfg(target_arch = "x86_64")]
extern "C" {
    pub fn do_syscall(ssn: u16, syscall_addr: u64, n_args: u32, ...) -> i32;
}
