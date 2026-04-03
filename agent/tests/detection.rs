//! Integration tests for the AegisOne detection engine.
//!
//! These tests run on all platforms (Linux CI + Windows release CI) and
//! validate the core detection logic without requiring OS-specific APIs.

use aegisone_agent::detection::{entropy, patterns};

// ── Entropy ────────────────────────────────────────────────────────────────────

mod entropy_tests {
    use super::*;

    #[test]
    fn zero_entropy_for_uniform_bytes() {
        let data = vec![0x00u8; 4096];
        let h = entropy::shannon(&data);
        assert!(h < 0.001, "uniform bytes should have ~0 entropy, got {h}");
    }

    #[test]
    fn max_entropy_for_all_256_values() {
        // Repeat all byte values enough times for a stable distribution
        let data: Vec<u8> = (0u8..=255).cycle().take(1024).collect();
        let h = entropy::shannon(&data);
        assert!(
            (h - 8.0).abs() < 0.01,
            "uniform distribution over all bytes should have entropy ≈ 8.0, got {h}"
        );
    }

    #[test]
    fn english_prose_is_moderate_entropy() {
        // Typical English text: 4–5 bits/byte
        let text = b"The quick brown fox jumps over the lazy dog. \
                     How vexingly quick daft zebras jump! \
                     Pack my box with five dozen liquor jugs.";
        let h = entropy::shannon(text);
        assert!(h > 3.5 && h < 6.0, "English text entropy out of expected range: {h}");
    }

    #[test]
    fn high_entropy_for_pseudo_random_data() {
        // LCG pseudo-random bytes look encrypted to the entropy checker
        let data: Vec<u8> = (0u32..8192)
            .scan(42u32, |state, _| {
                *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                Some((*state >> 16) as u8)
            })
            .collect();
        let h = entropy::shannon(&data);
        assert!(h > 7.0, "pseudo-random data should have entropy > 7.0, got {h}");
    }

    #[test]
    fn empty_slice_returns_zero() {
        assert_eq!(entropy::shannon(&[]), 0.0);
    }

    #[test]
    fn single_byte_value_is_zero_entropy() {
        let data = vec![0xFFu8; 512];
        assert!(entropy::shannon(&data) < 0.001);
    }

    #[test]
    fn two_values_equal_probability_is_one_bit() {
        // Alternating 0x00 / 0xFF → p(0)=0.5, p(1)=0.5 → H = 1.0
        let data: Vec<u8> = (0..1024).map(|i| if i % 2 == 0 { 0x00 } else { 0xFF }).collect();
        let h = entropy::shannon(&data);
        assert!(
            (h - 1.0).abs() < 0.01,
            "two equally-likely byte values should have entropy ≈ 1.0, got {h}"
        );
    }

    #[test]
    fn compressed_like_data_is_above_threshold() {
        // Deflate/zlib output looks like high-entropy data
        // Simulate with a sawtooth pattern (not truly random but higher entropy)
        let data: Vec<u8> = (0..=255u8).cycle().take(2048).collect();
        let h = entropy::shannon(&data);
        assert!(h > 7.9, "uniform byte distribution should be near max entropy, got {h}");
    }
}

// ── Patterns ───────────────────────────────────────────────────────────────────

mod pattern_tests {
    use super::*;

    // ── Ransomware extensions ──────────────────────────────────────────────────

    #[test]
    fn detects_known_ransomware_extensions() {
        let known = [
            ".locked", ".encrypted", ".crypt", ".ryk", ".clop",
            ".ransom", ".wcry", ".wncry", ".lck", ".cerber",
            ".locky", ".zepto", ".odin", ".micro", ".cryptolocker",
        ];
        for ext in known {
            assert!(
                patterns::is_ransomware_extension(ext, &[]),
                "should detect known extension: {ext}"
            );
        }
    }

    #[test]
    fn ignores_legitimate_extensions() {
        let legit = [
            ".txt", ".docx", ".xlsx", ".pdf", ".jpg", ".png",
            ".mp4", ".zip", ".exe", ".dll", ".rs", ".py",
        ];
        for ext in legit {
            assert!(
                !patterns::is_ransomware_extension(ext, &[]),
                "should NOT flag legitimate extension: {ext}"
            );
        }
    }

    #[test]
    fn case_insensitive_extension_matching() {
        assert!(patterns::is_ransomware_extension(".LOCKED", &[]));
        assert!(patterns::is_ransomware_extension(".Encrypted", &[]));
        assert!(patterns::is_ransomware_extension(".WNCRY", &[]));
    }

    #[test]
    fn custom_extension_list_is_respected() {
        let custom = vec![".payme".to_string(), ".pwned".to_string()];
        assert!(patterns::is_ransomware_extension(".payme", &custom));
        assert!(patterns::is_ransomware_extension(".pwned", &custom));
        assert!(!patterns::is_ransomware_extension(".payme", &[]));
    }

    // ── Ransom notes ───────────────────────────────────────────────────────────

    #[test]
    fn detects_known_ransom_note_filenames() {
        let notes = [
            "README.txt",
            "DECRYPT.txt",
            "HOW_TO_RECOVER.txt",
            "HOW_TO_DECRYPT.txt",
            "YOUR_FILES_ARE_ENCRYPTED.txt",
            "!!!RESTORE_FILES!!!.txt",
            "@WanaDecryptor@.exe",
        ];
        for note in notes {
            assert!(
                patterns::is_ransom_note(note, &[]),
                "should detect ransom note: {note}"
            );
        }
    }

    #[test]
    fn does_not_flag_normal_filenames() {
        let normal = ["report.txt", "budget.xlsx", "photo.jpg", "README.md"];
        for name in normal {
            assert!(
                !patterns::is_ransom_note(name, &[]),
                "should NOT flag normal file: {name}"
            );
        }
    }

    #[test]
    fn ransom_note_matching_is_case_insensitive() {
        assert!(patterns::is_ransom_note("readme.txt", &[]));
        assert!(patterns::is_ransom_note("decrypt.TXT", &[]));
    }

    // ── Shadow delete commands ─────────────────────────────────────────────────

    #[test]
    fn detects_vssadmin_delete() {
        let cmd = "vssadmin delete shadows /all /quiet";
        assert!(
            patterns::is_shadow_deletion_command(cmd),
            "should detect vssadmin delete: {cmd}"
        );
    }

    #[test]
    fn detects_wmic_shadowcopy_delete() {
        let cmd = "wmic shadowcopy delete";
        assert!(
            patterns::is_shadow_deletion_command(cmd),
            "should detect wmic shadowcopy delete: {cmd}"
        );
    }

    #[test]
    fn detects_bcdedit_recovery_disable() {
        let cmd = "bcdedit /set {default} recoveryenabled no";
        assert!(
            patterns::is_shadow_deletion_command(cmd),
            "should detect bcdedit recovery disable: {cmd}"
        );
    }

    #[test]
    fn detects_wbadmin_catalog_delete() {
        let cmd = "wbadmin delete catalog -quiet";
        assert!(
            patterns::is_shadow_deletion_command(cmd),
            "should detect wbadmin catalog delete: {cmd}"
        );
    }

    #[test]
    fn does_not_flag_innocent_commands() {
        assert!(!patterns::is_shadow_deletion_command("notepad.exe"));
        assert!(!patterns::is_shadow_deletion_command("vssadmin list shadows"));
        assert!(!patterns::is_shadow_deletion_command("cmd /c dir"));
    }

    // ── Random-looking extension heuristic ────────────────────────────────────

    #[test]
    fn flags_random_looking_extensions() {
        // Ransomware families that append a random hex-like extension
        assert!(patterns::looks_random_extension(".a3x8"));
        assert!(patterns::looks_random_extension(".b4f9c2"));
        assert!(patterns::looks_random_extension(".x7k2m1"));
    }

    #[test]
    fn does_not_flag_known_extensions_as_random() {
        assert!(!patterns::looks_random_extension(".docx"));
        assert!(!patterns::looks_random_extension(".pdf"));
        assert!(!patterns::looks_random_extension(".rs"));
        assert!(!patterns::looks_random_extension(".txt"));
    }

    #[test]
    fn short_or_long_extensions_not_flagged() {
        assert!(!patterns::looks_random_extension(".ab"));     // too short
        assert!(!patterns::looks_random_extension(".abcdefghij")); // too long
    }
}

// ── Template engine ────────────────────────────────────────────────────────────

mod template_tests {
    use aegisone_agent::report::template::Template;
    use std::collections::HashMap;

    #[test]
    fn renders_simple_variables() {
        let t = Template::new("Hello {{name}}, score is {{score}}.");
        let mut vars = HashMap::new();
        vars.insert("name", "Alice".to_string());
        vars.insert("score", "95".to_string());
        let out = t.render(&vars, &HashMap::new());
        assert_eq!(out, "Hello Alice, score is 95.");
    }

    #[test]
    fn missing_placeholder_becomes_dash() {
        let t = Template::new("Value: {{missing}}");
        let out = t.render(&HashMap::new(), &HashMap::new());
        assert_eq!(out, "Value: —");
    }

    #[test]
    fn multiple_occurrences_all_replaced() {
        let t = Template::new("{{x}} + {{x}} = ?");
        let mut vars = HashMap::new();
        vars.insert("x", "1".to_string());
        let out = t.render(&vars, &HashMap::new());
        assert_eq!(out, "1 + 1 = ?");
    }

    #[test]
    fn incident_report_template_loads() {
        // Verify the embedded template compiles and can be instantiated
        let t = Template::new(include_str!("../templates/incident_report.md"));
        let out = t.render(&HashMap::new(), &HashMap::new());
        // All {{placeholders}} should be replaced with —
        assert!(!out.contains("{{"));
    }
}
