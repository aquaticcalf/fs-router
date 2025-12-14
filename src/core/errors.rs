#[derive(Debug)]
pub enum RouteError {
    DuplicateRoute(String),
    InvalidGrammar(String),
    MultipleFallbacks,
}
