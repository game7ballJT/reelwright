//! Paths a web page may open from its own address (`?open=samples/title-sequence.fcproj`).
//!
//! The browser build (`apps/reelwright-web`) fetches the file with the page's origin, so a value
//! that names another site, or climbs out of the folder the page is served from, must never be
//! fetched: a link someone sends could otherwise make the page load a project of their choosing.

/// True for a short, relative path made of plain segments: no scheme or host, no query or
/// fragment, no backslashes, no `.` or `..` segments, no leading slash.
pub fn is_same_origin_path(p: &str) -> bool {
    !p.is_empty()
        && p.len() <= 200
        && !p.starts_with('/')
        && !p.contains([':', '\\', '?', '#', '%'])
        && !p.chars().any(char::is_control)
        && p.split('/').all(|s| !s.is_empty() && s != "." && s != "..")
}

#[cfg(test)]
mod tests {
    use super::is_same_origin_path as ok;

    #[test]
    fn plain_relative_paths_are_allowed() {
        assert!(ok("samples/title-sequence.fcproj"));
        assert!(ok("project.fcproj"));
        assert!(ok("a/b/c d.fcproj"));
    }

    #[test]
    fn other_sites_and_escapes_are_not() {
        for p in [
            "",
            "/etc/passwd",
            "//evil.example/x.fcproj",
            "https://evil.example/x.fcproj",
            "javascript:alert(1)",
            "data:text/plain,hi",
            "../x.fcproj",
            "samples/../../x.fcproj",
            "samples\\x.fcproj",
            "samples//x.fcproj",
            "./x.fcproj",
            "x.fcproj?y=1",
            "x.fcproj#frag",
            "%2e%2e/x.fcproj",
            "x\n.fcproj",
        ] {
            assert!(!ok(p), "{p:?}");
        }
        assert!(!ok(&"a/".repeat(120)));
    }
}
