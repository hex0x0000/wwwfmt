use std::path::PathBuf;

use oxc::allocator::Allocator;

use crate::{config::Config, css, files, html, javascript};

/// Formats one file. Used internally.
pub fn inner_file(
    path: &PathBuf,
    root: &Option<PathBuf>,
    ext: String,
    config: &Config,
    minify: bool,
    inplace: bool,
    alloc: &Allocator,
) -> Result<(), String> {
    let out_path = if inplace {
        None
    } else {
        Some(files::outdir(path.clone(), config, root, minify)?)
    };
    match ext.as_str() {
        "html" | "htm" => html::fmt(path, out_path, config, minify, alloc),
        "css" => css::fmt(path, out_path, config, minify),
        "js" | "mjs" | "jsx" | "cjs" | "ts" | "mts" | "cts" | "tsx" => {
            javascript::fmt(path, out_path, config, minify, alloc)
        }
        _ => Ok(()),
    }
}

/// Formats one file.
///
/// - [`path`]: file's original path.
/// - [`root`]: project's root dir. Needed if
pub fn file<P: Into<PathBuf>>(
    path: P,
    root: Option<P>,
    config: &Config,
    minify: bool,
    inplace: bool,
) -> Result<(), String> {
    let path: PathBuf = path.into();
    let ext = path
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_lowercase())
        .ok_or("Failed to get file extension.")?;
    inner_file(
        &path,
        &root.map(|p| p.into()),
        ext,
        config,
        minify,
        inplace,
        &Allocator::new(),
    )
}

/// Formats all files starting from the project's root directory.
pub fn all(root: PathBuf, config: &Config, minify: bool, inplace: bool) -> Result<(), String> {
    let alloc = Allocator::new();
    files::recurse_dir(&root, &Some(root.clone()), config, minify, inplace, &alloc)
}
