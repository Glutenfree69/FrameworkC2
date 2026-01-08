/// Macro d'obfuscation - hash compile-time des noms de fonctions
#[macro_export]
macro_rules! obf {
    ($s:expr) => {{
        static HASH: u32 = $crate::syscalls::obf::djb2_hash_str($s);
        HASH
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
        if cur >= ('a' as u8) {
            cur -= 0x20; // Uppercase
        }
        hsh = ((hsh << 5).wrapping_add(hsh)) + cur as u32;
        iter += 1;
    }
    hsh
}
