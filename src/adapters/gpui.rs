use crate::core::{ParamKind, RouteSpec, RouteTable};
use gpui::{AnyView, Context, IntoElement, Render, SharedString, Window, div, prelude::*, px, rgb};

#[derive(Debug, Clone)]
pub struct RouteMatch {
    pub spec: RouteSpec,
    pub params: Vec<(String, String)>,
}

pub struct RouterView {
    table: RouteTable<AnyView>,
    current_route: SharedString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigateError {
    RouteIdNotFound(u64),
    MissingParam { name: String },
    UnsupportedWildcard,
}

impl RouterView {
    pub fn new(table: RouteTable<AnyView>, initial_route: impl Into<SharedString>) -> Self {
        Self {
            table,
            current_route: initial_route.into(),
        }
    }

    pub fn route(&self) -> &str {
        &self.current_route
    }

    pub fn navigate(&mut self, route: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.current_route = route.into();
        cx.notify();
    }

    pub fn set_route(&mut self, route: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.navigate(route, cx);
    }

    pub fn navigate_by_id(
        &mut self,
        route_id: u64,
        params: &[(&str, &str)],
        cx: &mut Context<Self>,
    ) -> Result<(), NavigateError> {
        let (spec, _) = self
            .table
            .routes
            .iter()
            .find(|(spec, _)| spec.id == route_id)
            .ok_or(NavigateError::RouteIdNotFound(route_id))?;

        let route = build_path(spec, params)?;
        self.navigate(route, cx);
        Ok(())
    }

    pub fn table(&self) -> &RouteTable<AnyView> {
        &self.table
    }

    pub fn table_mut(&mut self) -> &mut RouteTable<AnyView> {
        &mut self.table
    }
}

impl Render for RouterView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let resolved = resolve_any_view(&self.table, &self.current_route);

        let (matched_view, matched_meta) = match resolved {
            Some(resolved) => {
                let view = Some(resolved.view);
                let meta = Some(resolved.matched);
                (view, meta)
            }
            None => (self.table.fallback.clone(), None),
        };

        let header = render_debug_header(&self.current_route, matched_meta.as_ref(), &self.table);

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(header)
            .child(
                div()
                    .flex_1()
                    .size_full()
                    .bg(rgb(0xffffff))
                    .when_some(matched_view, |d, view| d.child(view)),
            )
    }
}

struct ResolvedAnyView {
    matched: RouteMatch,
    view: AnyView,
}

fn resolve_any_view(table: &RouteTable<AnyView>, path: &str) -> Option<ResolvedAnyView> {
    let mut best: Option<(usize, RouteMatch, AnyView)> = None;

    for (spec, view) in &table.routes {
        let Some(matched) = match_route(spec, path) else {
            continue;
        };

        let score = score_spec(spec);
        match &best {
            Some((best_score, _, _)) if *best_score >= score => {}
            _ => best = Some((score, matched, view.clone())),
        }
    }

    best.map(|(_, matched, view)| ResolvedAnyView { matched, view })
}

fn score_spec(spec: &RouteSpec) -> usize {
    let tokens = tokenize_pattern(&spec.path, &spec.params);
    tokens
        .iter()
        .map(|t| match t {
            Token::Static(_) => 100,
            Token::ParamSingle(_) => 10,
            Token::ParamCatchAll(_) => 1,
            Token::Wildcard => 0,
        })
        .sum::<usize>()
        + tokens.len()
}

pub fn build_path(spec: &RouteSpec, params: &[(&str, &str)]) -> Result<String, NavigateError> {
    let tokens = tokenize_pattern(&spec.path, &spec.params);

    let mut result = String::new();
    for token in tokens {
        match token {
            Token::Static(s) => {
                if s != "/" {
                    result.push('/');
                    result.push_str(&s);
                }
            }
            Token::ParamSingle(name) => {
                let value = params
                    .iter()
                    .find(|(k, _)| *k == name)
                    .map(|(_, v)| *v)
                    .ok_or_else(|| NavigateError::MissingParam { name })?;

                result.push('/');
                result.push_str(value);
            }
            Token::ParamCatchAll(name) => {
                let value = params
                    .iter()
                    .find(|(k, _)| *k == name)
                    .map(|(_, v)| *v)
                    .ok_or_else(|| NavigateError::MissingParam { name })?;

                let value = value.trim_matches('/');
                if !value.is_empty() {
                    result.push('/');
                    result.push_str(value);
                }
            }
            Token::Wildcard => return Err(NavigateError::UnsupportedWildcard),
        }
    }

    if result.is_empty() {
        result.push('/');
    }

    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Static(String),
    ParamSingle(String),
    ParamCatchAll(String),
    Wildcard,
}

fn tokenize_pattern(pattern: &str, params: &[crate::core::ParamSpec]) -> Vec<Token> {
    let catchall_params: std::collections::HashSet<&str> = params
        .iter()
        .filter(|p| p.kind == ParamKind::CatchAll)
        .map(|p| p.name.as_str())
        .collect();

    let mut segments: Vec<&str> = pattern
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    if segments.is_empty() {
        segments.push("/");
    }

    let mut tokens = Vec::new();
    let mut i = 0;
    while i < segments.len() {
        let seg = segments[i];

        if seg == "*" {
            tokens.push(Token::Wildcard);
            i += 1;
            continue;
        }

        if let Some(name) = seg.strip_prefix(':') {
            let is_catchall = catchall_params.contains(name);

            if is_catchall && segments.get(i + 1).copied() == Some("*") {
                tokens.push(Token::ParamCatchAll(name.to_string()));
                i += 2;
                continue;
            }

            tokens.push(Token::ParamSingle(name.to_string()));
            i += 1;
            continue;
        }

        tokens.push(Token::Static(seg.to_string()));
        i += 1;
    }

    tokens
}

pub fn match_route(spec: &RouteSpec, path: &str) -> Option<RouteMatch> {
    let tokens = tokenize_pattern(&spec.path, &spec.params);

    let path_segments: Vec<&str> = path
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let mut params: Vec<(String, String)> = Vec::new();

    if match_tokens(&tokens, &path_segments, 0, 0, &mut params) {
        Some(RouteMatch {
            spec: spec.clone(),
            params,
        })
    } else {
        None
    }
}

fn match_tokens(
    tokens: &[Token],
    path_segments: &[&str],
    token_index: usize,
    path_index: usize,
    params: &mut Vec<(String, String)>,
) -> bool {
    if token_index == tokens.len() {
        return path_index == path_segments.len();
    }

    match &tokens[token_index] {
        Token::Static(expected) => {
            let Some(actual) = path_segments.get(path_index).copied() else {
                return false;
            };
            if actual != expected {
                return false;
            }
            match_tokens(tokens, path_segments, token_index + 1, path_index + 1, params)
        }
        Token::ParamSingle(name) => {
            let Some(value) = path_segments.get(path_index).copied() else {
                return false;
            };
            params.push((name.clone(), value.to_string()));
            let ok = match_tokens(tokens, path_segments, token_index + 1, path_index + 1, params);
            if !ok {
                params.pop();
            }
            ok
        }
        Token::ParamCatchAll(name) => {
            for end in (path_index + 1..=path_segments.len()).rev() {
                let captured = path_segments[path_index..end].join("/");
                params.push((name.clone(), captured));
                let ok = match_tokens(tokens, path_segments, token_index + 1, end, params);
                if ok {
                    return true;
                }
                params.pop();
            }
            false
        }
        Token::Wildcard => {
            if token_index + 1 == tokens.len() {
                return true;
            }

            for end in path_index..=path_segments.len() {
                if match_tokens(tokens, path_segments, token_index + 1, end, params) {
                    return true;
                }
            }
            false
        }
    }
}

fn render_debug_header(
    current_route: &str,
    matched: Option<&RouteMatch>,
    table: &RouteTable<AnyView>,
) -> impl IntoElement {
    let matched_path = matched.map(|m| m.spec.path.as_str()).unwrap_or("(none)");
    let matched_source = matched
        .map(|m| m.spec.source.as_str())
        .unwrap_or("(none)");

    let matched_kind = matched
        .map(|m| format!("{:?}", m.spec.kind))
        .unwrap_or_else(|| "(none)".to_string());

    let params_string = matched
        .map(|m| {
            if m.params.is_empty() {
                "(none)".to_string()
            } else {
                m.params
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        })
        .unwrap_or_else(|| "(none)".to_string());

    div()
        .flex()
        .flex_col()
        .gap_1()
        .p_2()
        .bg(rgb(0x161616))
        .text_color(rgb(0xf0f0f0))
        .text_sm()
        .child(format!("route: {current_route}"))
        .child(format!("matched: {matched_path}  kind: {matched_kind}"))
        .child(format!("params: {params_string}"))
        .child(format!("source: {matched_source}"))
        .child(format!(
            "routes: {}  fallback: {}",
            table.routes.len(),
            if table.fallback.is_some() { "yes" } else { "no" }
        ))
        .child(div().h(px(1.0)).bg(rgb(0x2a2a2a)))
}
