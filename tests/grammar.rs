use fs_router::core::errors::RouteError;
use fs_router::core::grammar::parse_file_path;
use fs_router::core::{ParamKind, ParamSpec, RouteKind};

fn id_for(inner: &str) -> u64 {
    inner.chars().map(|c| c as u64).sum()
}

#[test]
fn parses_static_index_to_root() {
    let spec = parse_file_path("pages/index.rs", None).unwrap();

    assert_eq!(spec.path, "/");
    assert_eq!(spec.kind, RouteKind::Static);
    assert!(spec.params.is_empty());
    assert_eq!(spec.source, "pages/index.rs");
    assert_eq!(spec.id, id_for("index"));
}

#[test]
fn parses_static_nested_path() {
    let spec = parse_file_path("pages/blog/post.rs", None).unwrap();

    assert_eq!(spec.path, "/blog/post");
    assert_eq!(spec.kind, RouteKind::Static);
    assert!(spec.params.is_empty());
    assert_eq!(spec.id, id_for("blog/post"));
}

#[test]
fn parses_dynamic_segment() {
    let spec = parse_file_path("pages/users/[id].rs", None).unwrap();

    assert_eq!(spec.path, "/users/:id");
    assert_eq!(spec.kind, RouteKind::Static);
    assert_eq!(
        spec.params,
        vec![ParamSpec {
            name: "id".to_string(),
            kind: ParamKind::Single
        }]
    );
}

#[test]
fn parses_multiple_dynamic_segments() {
    let spec = parse_file_path("pages/shop/[category]/[item_id].rs", None).unwrap();
    assert_eq!(spec.path, "/shop/:category/:item_id");
    assert_eq!(spec.kind, RouteKind::Static);
    assert_eq!(
        spec.params,
        vec![
            ParamSpec {
                name: "category".to_string(),
                kind: ParamKind::Single
            },
            ParamSpec {
                name: "item_id".to_string(),
                kind: ParamKind::Single
            }
        ]
    );
}

#[test]
fn parses_catchall_segment() {
    let spec = parse_file_path("pages/docs/[...slug].rs", None).unwrap();

    assert_eq!(spec.path, "/docs/:slug/*");
    assert_eq!(spec.kind, RouteKind::Static);
    assert_eq!(
        spec.params,
        vec![ParamSpec {
            name: "slug".to_string(),
            kind: ParamKind::CatchAll
        }]
    );
}

#[test]
fn parses_catchall_within_path() {
    let spec = parse_file_path("pages/files/[...filepath]/info.rs", None).unwrap();

    assert_eq!(spec.path, "/files/:filepath/*/info");
    assert_eq!(spec.kind, RouteKind::Static);
    assert_eq!(
        spec.params,
        vec![ParamSpec {
            name: "filepath".to_string(),
            kind: ParamKind::CatchAll
        }]
    );
}

#[test]
fn parses_fallback_404() {
    let spec = parse_file_path("pages/404.rs", None).unwrap();

    assert_eq!(spec.path, "//*");
    assert_eq!(spec.kind, RouteKind::Fallback);
    assert!(spec.params.is_empty());
    assert_eq!(spec.id, id_for("404"));
}

#[test]
fn honors_custom_page_dir() {
    let spec = parse_file_path("src_pages/api/health.rs", Some("src_pages")).unwrap();

    assert_eq!(spec.path, "/api/health");
    assert_eq!(spec.kind, RouteKind::Static);
    assert!(spec.params.is_empty());
}

#[test]
fn errors_when_not_in_page_dir() {
    let err = parse_file_path("other/index.rs", None).unwrap_err();

    match err {
        RouteError::InvalidGrammar(message) => {
            assert!(message.contains("invalid path"));
        }
        _ => panic!("expected InvalidGrammar"),
    }
}

#[test]
fn errors_when_not_rust_file() {
    let err = parse_file_path("pages/index.ts", None).unwrap_err();

    match err {
        RouteError::InvalidGrammar(message) => {
            assert!(message.contains("not a rust file"));
        }
        _ => panic!("expected InvalidGrammar"),
    }
}
