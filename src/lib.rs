mod config;
mod css;
mod files;
mod fmt;
mod html;
mod javascript;

pub use fmt::{all, file};

pub mod conf {
    pub use super::config::{Config, Css, Html, IndentKind, JavaScript};
}
