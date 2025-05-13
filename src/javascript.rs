use std::path::PathBuf;

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions},
    minifier::{CompressOptions, MangleOptions, Minifier, MinifierOptions},
    parser::{Parser, ParserReturn},
    span::SourceType,
};

use crate::{config::Config, files};

pub fn fmt_str(
    src: &str,
    src_type: SourceType,
    alloc: &Allocator,
    config: &Config,
    minify: bool,
) -> Result<String, String> {
    let ParserReturn {
        mut program,
        errors: parser_errors,
        panicked,
        ..
    } = Parser::new(alloc, src, src_type).parse();
    if panicked {
        let errors: Vec<String> = parser_errors.into_iter().map(|e| e.to_string()).collect();
        return Err(errors.join("\t\n"));
    }
    if minify {
        let minifier = Minifier::new(MinifierOptions {
            mangle: if config.javascript.uglify_mangle {
                Some(MangleOptions::default())
            } else {
                None
            },
            compress: Some(CompressOptions {
                drop_debugger: config.javascript.uglify_drop_debugger,
                drop_console: config.javascript.uglify_drop_console,
                ..CompressOptions::default()
            }),
        })
        .build(alloc, &mut program);
        Ok(Codegen::new()
            .with_options(CodegenOptions {
                minify: true,
                single_quote: config.javascript.use_single_quotes,
                comments: config.javascript.uglify_remove_comments,
                ..CodegenOptions::default()
            })
            .with_scoping(minifier.scoping)
            .build(&program)
            .code)
    } else {
        Ok(Codegen::new()
            .with_options(CodegenOptions {
                minify: false,
                single_quote: config.javascript.use_single_quotes,
                comments: true,
                ..Default::default()
            })
            .build(&program)
            .code)
    }
}

pub fn fmt(
    path: &PathBuf,
    out_path: Option<PathBuf>,
    config: &Config,
    minify: bool,
    alloc: &Allocator,
) -> Result<(), String> {
    let file = files::read(path)?;
    let fmted = fmt_str(
        &file,
        SourceType::from_path(path).unwrap_or_else(|e| panic!("Unknown javascript extension: {e}")),
        alloc,
        config,
        minify,
    )?;
    files::write(path, out_path, fmted.as_bytes())
}
