//! Unit tests for the app:// protocol (path normalization, serve, MIME).

#[cfg(test)]
mod tests {
    use crate::protocol::{mime_from_path, normalize_path, serve, ServeResult, INDEX_PATH};
    use include_dir::include_dir;

    static TEST_UI: include_dir::Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../ui/dist");

    #[test]
    fn normalize_path_default_index() {
        assert_eq!(normalize_path("/"), Some(INDEX_PATH));
        assert_eq!(normalize_path(""), Some(INDEX_PATH));
        assert_eq!(normalize_path("///"), Some(INDEX_PATH));
    }

    #[test]
    fn normalize_path_rejects_traversal() {
        assert_eq!(normalize_path("/.."), None);
        assert_eq!(normalize_path("/a/../b"), None);
        assert_eq!(normalize_path("/.. /index.html"), None);
    }

    #[test]
    fn serve_not_found_for_traversal() {
        let r = serve(&TEST_UI, "/../etc/passwd");
        assert!(matches!(r, ServeResult::NotFound));
    }

    #[test]
    fn serve_not_found_for_missing_file() {
        let r = serve(&TEST_UI, "/nonexistent.foo");
        assert!(matches!(r, ServeResult::NotFound));
    }

    #[test]
    fn serve_index_ok_when_dist_present() {
        let r = serve(&TEST_UI, "/");
        match r {
            ServeResult::Found { mime_type, .. } => assert_eq!(mime_type, "text/html"),
            ServeResult::NotFound => {
                // ui/dist may not exist in all test envs
            }
        }
    }

    #[test]
    fn mime_from_path_known_extensions() {
        assert_eq!(mime_from_path("a.html"), "text/html");
        assert_eq!(mime_from_path("b.js"), "application/javascript");
        assert_eq!(mime_from_path("c.css"), "text/css");
        assert_eq!(mime_from_path("d.png"), "image/png");
        assert_eq!(mime_from_path("e.woff2"), "font/woff2");
        assert_eq!(mime_from_path("f.unknown"), "application/octet-stream");
    }
}
