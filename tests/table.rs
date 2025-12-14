use fs_router::{RouteError, RouteKind, RouteSpec, RouteTable};

fn spec(path: &str, kind: RouteKind) -> RouteSpec {
    RouteSpec {
        id: 0,
        path: path.to_string(),
        kind,
        params: vec![],
        source: "".to_string(),
    }
}

#[test]
fn insert_rejects_duplicate_paths() {
    let mut table = RouteTable::new();

    table.insert(spec("/users/:id", RouteKind::Static), 1).unwrap();
    let err = table.insert(spec("/users/:id", RouteKind::Static), 2).unwrap_err();

    match err {
        RouteError::DuplicateRoute(path) => assert_eq!(path, "/users/:id"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn insert_rejects_multiple_fallbacks() {
    let mut table = RouteTable::new();

    table.insert(spec("//*", RouteKind::Fallback), 1).unwrap();
    let err = table.insert(spec("//*", RouteKind::Fallback), 2).unwrap_err();

    match err {
        RouteError::MultipleFallbacks => {}
        other => panic!("unexpected error: {other:?}"),
    }
}
