pub fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .eq_ignore_ascii_case("wayland")
}

pub fn is_snap() -> bool {
    !std::env::var("SNAP").unwrap_or_default().is_empty()
}