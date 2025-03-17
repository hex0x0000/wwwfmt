use crate::files::{get_currdir, revtraverse};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub enum IndentKind {
    Tab,
    Space,
}

#[derive(Serialize, Deserialize)]
pub struct Html {
    /// Indent character used when indenting. Valid when prettifying
    pub prettify_indent_kind: IndentKind,
    /// How many times the indent char is repeated. Valid when prettifying
    pub prettify_indent_num: usize,
    /// Tags whose children are not indented (by default all inline tags are listed). Valid when
    /// prettifying
    pub prettify_noindent_tags: Vec<String>,
    /// Remove comments from HTML. Valid when minifying
    pub uglify_rm_comments: bool,
    /// Format tag's attributes
    pub fmt_attrs: bool,
}

impl Default for Html {
    fn default() -> Self {
        Self {
            prettify_indent_kind: IndentKind::Space,
            prettify_indent_num: 2,
            prettify_noindent_tags: vec![
                "a", "span", "b", "i", "em", "strong", "del", "sup", "sub", "ins", "bdi", "bdo",
                "cite", "code", "data", "kbd", "mark", "q", "rp", "rt", "ruby", "s", "samp",
                "small", "time", "u", "var",
            ]
            .into_iter()
            .map(|s| s.to_owned())
            .collect(),
            uglify_rm_comments: true,
            fmt_attrs: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Css {
    /// Optimize CSS while minifying
    pub uglify_optimize: bool,
}

impl Default for Css {
    fn default() -> Self {
        Self {
            uglify_optimize: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JavaScript {
    /// Use only single quotes. Valid for both minifying a prettifying
    pub use_single_quotes: bool,
    /// Remove comments when minifying
    pub uglify_remove_comments: bool,
    /// Mangle identifiers when minifying. Reduces notably code's size but it makes it more
    /// obfuscated
    pub uglify_mangle: bool,
    /// Drop debugger calls in code when minifying
    pub uglify_drop_debugger: bool,
    /// Drop console calls in code when minifying
    pub uglify_drop_console: bool,
}

impl Default for JavaScript {
    fn default() -> Self {
        Self {
            use_single_quotes: true,
            uglify_remove_comments: true,
            uglify_mangle: true,
            uglify_drop_debugger: false,
            uglify_drop_console: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub html: Html,
    pub css: Css,
    pub javascript: JavaScript,
    /// Output directory of minified files
    pub uglify_outdir: Option<String>,
    /// Output directory of prettified files
    pub prettify_outdir: Option<String>,
    /// Ignore these paths
    pub ignore_path: Vec<String>,
}

impl Default for Config {
    /// Returns the default configuration
    fn default() -> Self {
        Config {
            html: Html::default(),
            css: Css::default(),
            javascript: JavaScript::default(),
            uglify_outdir: Some("wwwugly".into()),
            prettify_outdir: None,
            ignore_path: vec!["wwwugly"].into_iter().map(|s| s.to_owned()).collect(),
        }
    }
}

impl Config {
    /// Finds the configuration file, opens it and returns the parsed file with the directory
    pub fn find() -> Result<(Self, PathBuf), String> {
        let path = revtraverse(get_currdir()?, ".wwwfmt.toml")
            .map_err(|e| format!("Failed to find .wwwfmt.toml: {e}"))?;
        Ok((Self::open(&path)?, path))
    }

    /// Opens config file from path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut buf = String::new();
        File::open(path)
            .map_err(|e| format!("Failed to open config file: {e}"))?
            .read_to_string(&mut buf)
            .map_err(|e| format!("Failed to read config file: {e}"))?;
        toml::from_str(&buf).map_err(|e| format!("Config error: {e}"))
    }

    /// Writes the default configuration to the current working directory
    pub fn write_default() -> Result<(), String> {
        let mut path = get_currdir()?;
        path.push(".wwwfmt.toml");
        let config =
            toml::to_string(&Self::default()).expect("Default config serialization failed");
        File::create(path)
            .map_err(|e| format!("Failed to create file: {e}"))?
            .write_all(config.as_bytes())
            .map_err(|e| format!("Failed to write config: {e}"))?;
        Ok(())
    }
}

impl IndentKind {
    pub fn to_char(&self) -> char {
        match self {
            Self::Space => ' ',
            Self::Tab => '\t',
        }
    }

    pub fn repeat(&self, n: usize) -> String {
        [self.to_char()].repeat(n).into_iter().collect()
    }
}
