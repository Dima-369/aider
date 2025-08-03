pub fn normalize_display_path<S: AsRef<str>>(s: S) -> String {
    let mut display = s.as_ref().to_string();
    if let Some(stripped) = display.strip_prefix("./") {
        display = stripped.to_string();
    }
    display
}
