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
/// - `path`: file's original path.
/// - `root`: project's root dir. Needed if files need to be outputted in an outdir.
/// - `config`: Configuration
/// - `minify`: Whether to minify or prettify
/// - `inplace`: Whether to format in-place or in another file. If the output directory is
/// specified in the confi directory, the output file will end up there, otherwise a new file
/// called either file.prty.ext or file.min.ext will be created on the same directory.
/// - `alloc`: Oxc's arena [`Allocator`]. You can optionally specify this to improve performance
/// if you are manually iterating files. (using [`all`] is recommended)
///
/// The file's type is automatically recognized by its extension
pub fn file<P: Into<PathBuf>>(
    path: P,
    root: Option<P>,
    config: &Config,
    minify: bool,
    inplace: bool,
    alloc: Option<&Allocator>,
) -> Result<(), String> {
    let path: PathBuf = path.into();
    let ext = path
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_lowercase())
        .ok_or("Failed to get file extension.")?;
    let alloc = if let Some(alloc) = alloc {
        alloc
    } else {
        &Allocator::new()
    };
    inner_file(
        &path,
        &root.map(|p| p.into()),
        ext,
        config,
        minify,
        inplace,
        alloc,
    )
}

/// Formats all files starting from the project's root directory.
///
/// The file's type are automatically recognized by their extension, if an extension is not
/// recognized the file is skipped.
pub fn all<P: Into<PathBuf>>(
    root: P,
    config: &Config,
    minify: bool,
    inplace: bool,
) -> Result<(), String> {
    let alloc = Allocator::new();
    let root: PathBuf = root.into();
    files::recurse_dir(&root, &Some(root.clone()), config, minify, inplace, &alloc)
}
