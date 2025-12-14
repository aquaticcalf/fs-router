use super::errors::RouteError;
use super::spec::{ParamKind, ParamSpec, RouteKind, RouteSpec};

pub fn parse_file_path(path: &str, page_dir: Option<&str>) -> Result<RouteSpec, RouteError> {
    let page_dir = match page_dir {
        Some(dir) => dir,
        None => "pages",
    };

     let inner = path
        .strip_prefix(&format!("{}/", page_dir))
        .ok_or_else(|| RouteError::InvalidGrammar(format!("invalid path : {}", path)))?;

     let inner = inner
        .strip_suffix(".rs")
        .ok_or_else(|| RouteError::InvalidGrammar(format!("not a rust file : {}", path)))?;

    let segments: Vec<&str> = inner.split('/').collect();
    let mut route_path = String::new();
    let mut params = Vec::new();
    let mut kind = RouteKind::Static;

    for segment in &segments {
        if segment.is_empty() {
            continue;
        }
        
        route_path.push('/');

        if *segment == "index" && segments.len() == 1 {
            // static index
        }

        else if let Some(name) = segment.strip_prefix("[...").and_then(|s| s.strip_suffix("]")) {
            // catchall [...name]
            route_path.push(':');
            route_path.push_str(name);
            route_path.push_str("/*");
            params.push(ParamSpec { name: name.to_string(), kind: ParamKind::CatchAll });
        }

        else if let Some(name) = segment.strip_prefix("[").and_then(|s| s.strip_suffix("]")) {
            // dynamic [name]
            route_path.push(':');
            route_path.push_str(name);
            params.push(ParamSpec { name: name.to_string(), kind: ParamKind::Single });
        }

        else if *segment == "404" {
            // fallback
            kind = RouteKind::Fallback;
            route_path.push_str("/*");
        }

        else {
            // static segment
            route_path.push_str(segment);
        }
    }

    if route_path.is_empty() {
        route_path.push_str("/");
    }

    let id = inner.chars().map(|c| c as u64).sum();

    Ok(RouteSpec {
        id,
        path: route_path,
        kind,
        params,
        source: path.to_string(),
    })
}
