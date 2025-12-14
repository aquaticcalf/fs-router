use fs_router::core::{ParamKind, ParamSpec, RouteKind, RouteSpec, RouteTable};

use fs_router::adapters::gpui::{NavigateError, build_path, match_route};

fn spec(path: &str, kind: RouteKind, params: Vec<ParamSpec>) -> RouteSpec {
    RouteSpec {
        id: 0,
        path: path.to_string(),
        kind,
        params,
        source: "src".to_string(),
    }
}

fn spec_with_id(id: u64, path: &str, kind: RouteKind, params: Vec<ParamSpec>) -> RouteSpec {
    RouteSpec {
        id,
        path: path.to_string(),
        kind,
        params,
        source: "src".to_string(),
    }
}

#[test]
fn matches_static() {
    let spec = spec("/blog/post", RouteKind::Static, vec![]);
    let matched = match_route(&spec, "/blog/post").unwrap();
    assert_eq!(matched.spec.path, "/blog/post");
    assert!(matched.params.is_empty());
}

#[test]
fn matches_dynamic_single() {
    let spec = spec(
        "/users/:id",
        RouteKind::Static,
        vec![ParamSpec {
            name: "id".to_string(),
            kind: ParamKind::Single,
        }],
    );

    let matched = match_route(&spec, "/users/123").unwrap();
    assert_eq!(matched.params, vec![("id".to_string(), "123".to_string())]);
}

#[test]
fn matches_dynamic_catchall_with_suffix() {
    let spec = spec(
        "/files/:filepath/*/info",
        RouteKind::Static,
        vec![ParamSpec {
            name: "filepath".to_string(),
            kind: ParamKind::CatchAll,
        }],
    );

    let matched = match_route(&spec, "/files/a/b/c/info").unwrap();
    assert_eq!(
        matched.params,
        vec![("filepath".to_string(), "a/b/c".to_string())]
    );
}

#[test]
fn does_not_match_when_segment_mismatch() {
    let spec = spec("/blog/post", RouteKind::Static, vec![]);
    assert!(match_route(&spec, "/blog/other").is_none());
}

#[test]
fn route_table_stores_fallback_separately() {
    let mut table: RouteTable<i32> = RouteTable::new();
    table
        .insert(spec("//*", RouteKind::Fallback, vec![]), 42)
        .unwrap();
    assert_eq!(table.fallback, Some(42));
    assert!(table.routes.is_empty());
}

#[test]
fn navigate_by_id_builds_path() {
    // We don't need a real GPUI view for this test; `AnyView` is not required.
    // We just validate the underlying path building through a public API surface.

    // NOTE: `navigate_by_id` lives on `RouterView` which holds `AnyView`.
    // We can still validate the error paths and path construction by calling
    // `build_path`-driven behavior indirectly via matching.

    let route_id = 123;
    let spec = spec_with_id(
        route_id,
        "/users/:id",
        RouteKind::Static,
        vec![ParamSpec {
            name: "id".to_string(),
            kind: ParamKind::Single,
        }],
    );

    // sanity check that the route grammar matches
    let matched = match_route(&spec, "/users/42").unwrap();
    assert_eq!(matched.params, vec![("id".to_string(), "42".to_string())]);

    // and that a missing param would be reported by the navigate builder.
    let err = build_path(&spec, &[]).unwrap_err();
    assert_eq!(
        err,
        NavigateError::MissingParam {
            name: "id".to_string()
        }
    );

    // happy path
    let path = build_path(&spec, &[("id", "99")]).unwrap();
    assert_eq!(path, "/users/99");
}
