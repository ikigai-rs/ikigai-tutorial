//! Keep the shared view language and HTTP transport on one version across the books.

#[test]
fn the_books_share_one_compose_and_http_implementation() {
    let lock = include_str!("../../../Cargo.lock");
    for name in ["ikigai-fn", "ikigai-http"] {
        let versions: Vec<_> = lock
            .split("[[package]]")
            .filter(|package| {
                package
                    .lines()
                    .any(|line| line == format!("name = \"{name}\""))
            })
            .flat_map(|package| {
                package
                    .lines()
                    .filter(|line| line.starts_with("version = "))
            })
            .collect();
        assert_eq!(
            versions.len(),
            1,
            "{name} must have one implementation: {versions:?}"
        );
    }
}
