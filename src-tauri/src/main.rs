#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sshtunnel_app_lib::run();
}

#[cfg(test)]
mod tests {
    #[test]
    fn windows_release_build_uses_gui_subsystem() {
        let source = include_str!("main.rs");
        assert!(
            source.contains("windows_subsystem = \"windows\""),
            "expected Windows GUI subsystem attribute in main.rs"
        );
    }
}
