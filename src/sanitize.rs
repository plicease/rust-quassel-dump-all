/// Turns an arbitrary network/channel name into something safe to use as a
/// single path component on common filesystems.
pub fn sanitize_component(name: &str) -> String {
    let mut out: String = name
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if (c as u32) < 0x20 => '_',
            c => c,
        })
        .collect();
    while out.ends_with('.') || out.ends_with(' ') {
        out.pop();
    }
    if out.is_empty() || out == "." || out == ".." {
        out = "buffer".to_string();
    }
    out
}

/// Picks a filename for `name` (already sanitized as a component, extension
/// appended) that doesn't collide with anything already in `used`.
pub fn unique_filename(
    used: &mut std::collections::HashSet<String>,
    name: &str,
    extension: &str,
) -> String {
    let base = sanitize_component(name);
    let mut candidate = format!("{base}.{extension}");
    let mut n = 2;
    while used.contains(&candidate) {
        candidate = format!("{base}_{n}.{extension}");
        n += 1;
    }
    used.insert(candidate.clone());
    candidate
}
