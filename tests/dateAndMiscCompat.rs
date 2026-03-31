mod compat_misc {
    use std::path::PathBuf;
    use std::process::Command;

    fn proteus_binary() -> String {
        std::env::var("CARGO_BIN_EXE_proteus").unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("debug")
                .join("proteus")
                .display()
                .to_string()
        })
    }

    #[test]
    fn help_and_posix_info_cover_new_coreutils_and_misc_commands() {
        let printf_help = Command::new(proteus_binary())
            .args(["printf", "--help"])
            .output()
            .unwrap();
        assert!(printf_help.status.success());
        assert!(String::from_utf8_lossy(&printf_help.stdout).contains("printf"));

        let date_info = Command::new(proteus_binary())
            .args(["--posix-info", "date"])
            .output()
            .unwrap();
        assert!(date_info.status.success());
        assert!(String::from_utf8_lossy(&date_info.stdout).contains("feature_flag: date"));
    }
}
