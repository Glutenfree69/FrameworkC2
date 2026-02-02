/// Macro d'obfuscation - hash compile-time des noms de fonctions
#[macro_export]
macro_rules! obf {
    ($s:expr) => {{
        static HASH: u32 = $crate::syscalls::obf::djb2_hash_str($s);
        HASH
    }};
}

// ============================================================
// STRING OBFUSCATION - XOR compile-time, decrypt runtime
// ============================================================
// Clé XOR pour obfusquer les strings (changez-la pour votre build)
pub const XOR_KEY: u8 = 0x42;

/// Chiffre une string au compile-time avec XOR
/// Retourne un tableau de bytes chiffrés
pub const fn xor_encrypt<const N: usize>(input: &[u8]) -> [u8; N] {
    let mut out = [0u8; N];
    let mut i = 0;
    while i < N && i < input.len() {
        out[i] = input[i] ^ XOR_KEY;
        i += 1;
    }
    out
}

/// Déchiffre un tableau de bytes XOR au runtime
#[inline(always)]
pub fn xor_decrypt(encrypted: &[u8]) -> Vec<u8> {
    encrypted.iter().map(|b| b ^ XOR_KEY).collect()
}

/// Macro pour strings obfusquées - chiffrement compile-time, déchiffrement runtime
/// Usage: obf_str!("MaString") -> String
#[macro_export]
macro_rules! obf_str {
    ($s:expr) => {{
        // Chiffrement au compile-time
        const INPUT: &[u8] = $s.as_bytes();
        const LEN: usize = INPUT.len();
        const ENCRYPTED: [u8; LEN] = $crate::syscalls::obf::xor_encrypt::<LEN>(INPUT);
        
        // Déchiffrement au runtime
        String::from_utf8_lossy(&$crate::syscalls::obf::xor_decrypt(&ENCRYPTED)).into_owned()
    }};
}

/// Hash DJB2 d'une string (compile-time)
pub const fn djb2_hash_str(arg: &str) -> u32 {
    djb2_hash(arg.as_bytes())
}

/// Hash DJB2 d'un buffer (compile-time)
pub const fn djb2_hash(buffer: &[u8]) -> u32 {
    let mut hsh: u32 = 5381;
    let mut iter: usize = 0;
    let mut cur: u8;

    while iter < buffer.len() {
        cur = buffer[iter];
        if cur == 0 {
            iter += 1;
            continue;
        }
        if cur >= b'a' {
            cur -= 0x20; // Uppercase
        }
        hsh = ((hsh << 5).wrapping_add(hsh)) + cur as u32;
        iter += 1;
    }
    hsh
}
