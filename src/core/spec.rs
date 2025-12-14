#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteKind {
    Static,
    Dynamic,
    CatchAll,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamKind {
    Single,
    CatchAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamSpec {
    pub name: String,
    pub kind: ParamKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteSpec {
    pub id: u64,
    pub path: String,
    pub kind: RouteKind,
    pub params: Vec<ParamSpec>,
    pub source: String,
}
