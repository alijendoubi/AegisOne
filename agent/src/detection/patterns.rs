/// Returns true if the file extension (lowercased, with leading dot) is a
/// known ransomware output extension.
pub fn is_ransomware_extension(ext: &str, extra: &[String]) -> bool {
    const KNOWN: &[&str] = &[
        ".locked", ".encrypted", ".crypt", ".enc",
        ".ryk", ".ryuk",
        ".clop",
        ".ransom",
        ".pay2me",
        ".wcry", ".wncry", ".wncryt",
        ".lck",
        ".cerber", ".cerber2", ".cerber3",
        ".locky", ".zepto", ".odin", ".aesir", ".shit", ".thor",
        ".micro",
        ".cryptolocker", ".cryptowall",
        ".crypz", ".cryp1",
        ".darkness",
        ".breaking_bad",
        ".ecc", ".ezz", ".exx",
        ".vvv", ".abc", ".xyz", ".zzz",
        ".aaa", ".bbb", ".ccc",
        ".porno", ".btc",
        ".encoderpass",
        ".magic",
        ".revenge",
        ".wallet",
        ".onion",
        ".777",
    ];

    let lower = ext.to_lowercase();
    KNOWN.contains(&lower.as_str()) || extra.iter().any(|e| e.to_lowercase() == lower)
}

/// Returns true if the filename looks like a ransomware ransom note.
pub fn is_ransom_note(filename: &str, extra: &[String]) -> bool {
    const KNOWN: &[&str] = &[
        "README.txt",
        "DECRYPT.txt",
        "DECRYPT_INSTRUCTIONS.txt",
        "HOW_TO_RECOVER.txt",
        "HOW_TO_DECRYPT.txt",
        "HOW_TO_BUY.txt",
        "RECOVERY.txt",
        "READ_ME.txt",
        "HELP_DECRYPT.txt",
        "HELP_TO_SAVE_FILES.txt",
        "YOUR_FILES_ARE_ENCRYPTED.txt",
        "RANSOM_NOTE.txt",
        "!!!RESTORE_FILES!!!.txt",
        "!!! README !!!.txt",
        "_RECOVER_INSTRUCTIONS.txt",
        "@DECRYPT.txt",
        "@PLEASE_READ_ME@.txt",
        "WannaDecryptor.exe",
        "WanaDecryptor.exe",
        "@WanaDecryptor@.exe",
        "DECRYPT_INSTRUCTION.HTML",
        "HELP_RESTORE_FILES.txt",
    ];

    let upper = filename.to_uppercase();
    KNOWN.iter().any(|n| n.to_uppercase() == upper)
        || extra.iter().any(|n| n.to_uppercase() == upper)
}

/// Returns true if the command line contains known shadow-deletion patterns.
/// Matches ATT&CK T1490 (Inhibit System Recovery).
pub fn is_shadow_deletion_command(cmdline: &str) -> bool {
    let lower = cmdline.to_lowercase();
    (lower.contains("vssadmin") && lower.contains("delete"))
        || (lower.contains("wmic") && lower.contains("shadowcopy") && lower.contains("delete"))
        || (lower.contains("bcdedit") && lower.contains("recoveryenabled") && lower.contains("no"))
        || (lower.contains("wbadmin") && lower.contains("delete") && lower.contains("catalog"))
}

/// Returns true if the extension looks randomly generated (entropy-based heuristic).
/// Random extensions like `.a3x8` used by some novel ransomware families.
pub fn looks_random_extension(ext: &str) -> bool {
    let s = ext.trim_start_matches('.');
    if s.len() < 4 || s.len() > 10 {
        return false;
    }
    // All alphanumeric and no common English patterns
    if !s.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }
    // Reject known legitimate short extensions
    const LEGIT: &[&str] = &[
        "txt", "doc", "docx", "xls", "xlsx", "pdf", "ppt", "pptx",
        "jpg", "jpeg", "png", "gif", "bmp", "tiff",
        "mp3", "mp4", "wav", "avi", "mkv",
        "zip", "rar", "7z", "gz", "tar",
        "exe", "dll", "sys", "bat", "cmd", "ps1",
        "html", "htm", "css", "js", "ts", "json", "xml", "yaml", "toml",
        "rs", "py", "go", "java", "cpp", "c", "h",
        "csv", "log", "cfg", "ini",
        "bak", "tmp", "lock",
    ];
    !LEGIT.contains(&s.to_lowercase().as_str())
}
