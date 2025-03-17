use std::path::PathBuf;

use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
};

use crate::{config::Config, files};

pub fn fmt_str(src: &str, config: &Config, minify: bool) -> Result<String, String> {
    let mut stylesheet = StyleSheet::parse(src, ParserOptions::default())
        .map_err(|e| format!("Failed to parse CSS: {e}"))?;
    if minify && config.css.uglify_optimize {
        stylesheet
            .minify(MinifyOptions::default())
            .map_err(|e| format!("Failed to optimize CSS: {e}"))?;
    }
    Ok(stylesheet
        .to_css(PrinterOptions {
            minify,
            ..PrinterOptions::default()
        })
        .map_err(|e| format!("Failed to minify CSS: {e}"))?
        .code)
}

pub fn fmt(
    path: &PathBuf,
    out_path: Option<PathBuf>,
    config: &Config,
    minify: bool,
) -> Result<(), String> {
    let file = files::read(path)?;
    let fmted = fmt_str(&file, config, minify)?;
    files::write(path, out_path, fmted.as_bytes())
}
