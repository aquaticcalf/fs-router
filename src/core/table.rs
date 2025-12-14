use std::collections::HashSet;

use super::errors::RouteError;
use super::spec::{RouteKind, RouteSpec};

#[derive(Debug, Clone)]
pub struct RouteTable<T> {
    pub routes: Vec<(RouteSpec, T)>,
    pub fallback: Option<T>,
}

impl<T> RouteTable<T> {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            fallback: None,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            routes: Vec::with_capacity(capacity),
            fallback: None,
        }
    }

    pub fn insert(&mut self, spec: RouteSpec, handler: T) -> Result<(), RouteError> {
        if spec.kind == RouteKind::Fallback {
            if self.fallback.is_some() {
                return Err(RouteError::MultipleFallbacks);
            }
            self.fallback = Some(handler);
            return Ok(());
        }

        if self.routes.iter().any(|(existing, _)| existing.path == spec.path) {
            return Err(RouteError::DuplicateRoute(spec.path));
        }

        self.routes.push((spec, handler));
        Ok(())
    }

    pub fn from_routes<I>(routes: I) -> Result<Self, RouteError>
    where
        I: IntoIterator<Item = (RouteSpec, T)>,
    {
        let mut table = Self::new();
        let mut seen_paths: HashSet<String> = HashSet::new();

        for (spec, handler) in routes {
            if spec.kind == RouteKind::Fallback {
                if table.fallback.is_some() {
                    return Err(RouteError::MultipleFallbacks);
                }
                table.fallback = Some(handler);
                continue;
            }

            if !seen_paths.insert(spec.path.clone()) {
                return Err(RouteError::DuplicateRoute(spec.path));
            }

            table.routes.push((spec, handler));
        }

        Ok(table)
    }
}
