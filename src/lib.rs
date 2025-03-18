#[doc = include_str!("../README.md")]

mod config;
mod css;
mod files;
mod fmt;
mod html;
mod javascript;

// Re-export Oxc for the allocator
pub use oxc;

pub use fmt::{all, file};

/// Configuration options
pub mod conf {
    pub use super::config::{Config, Css, Html, IndentKind, JavaScript};
}
