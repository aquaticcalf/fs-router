pub mod core;

pub mod adapters;

pub use core::{
    ParamSpec,
    RouteKind,
    RouteSpec,
    RouteTable,
};

pub use core::errors::RouteError;
pub use core::grammar::parse_file_path;
pub use core::scan::scan_pages;

pub use adapters::gpui::{
    RouterView,
    NavigateError,
    RouteMatch
};
