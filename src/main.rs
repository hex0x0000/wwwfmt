mod config;
mod css;
mod files;
mod fmt;
mod html;
mod javascript;
#[cfg(test)]
mod tests;

use std::env;
use std::process::ExitCode;

use argh::FromArgs;
use config::Config;

#[derive(FromArgs, Clone)]
#[argh(help_triggers("-h", "--help"))]
/// Format HTML/CSS/JS files
struct Cli {
    /// minify files
    #[argh(switch, short = 'u')]
    uglify: bool,
    /// prettify files
    #[argh(switch, short = 'p')]
    prettify: bool,
    /// specify config file (by default wwwfmt tries to find it)
    #[argh(option, short = 'c')]
    cfg: Option<String>,
    /// write default configuration
    #[argh(switch)]
    write_default: bool,
    /// use default configuration
    #[argh(switch, short = 'd')]
    default_cfg: bool,
    /// minifies all files starting from the root of the project (where the .wwwfmt.toml file lies) into the wwwmin directoy
    #[argh(switch, short = 'a')]
    all: bool,
    /// minifies a single file
    #[argh(option, short = 'f')]
    file: Option<String>,
    /// minifies and replaces the file(s) in place
    #[argh(switch)]
    inplace: bool,
    /// prettifies file(s) in new file(s)
    #[argh(switch)]
    no_inplace: bool,
}

fn main() -> ExitCode {
    let cmd: Cli = argh::from_env();
    match handle(cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            println!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn handle(cmd: Cli) -> Result<(), String> {
    if cmd.write_default {
        Config::write_default()?;
        return Ok(());
    }

    // Get minify/inplace
    let (minify, inplace) = match (cmd.uglify, cmd.prettify) {
        (true, false) => (true, cmd.inplace),
        (false, true) => (false, !cmd.no_inplace),
        _ => return Err("You must choose either --uglify or --minify".into()),
    };

    // Get config
    let cfg = if let Some(path) = &cmd.cfg {
        Config::open(path).map_err(|e| format!("Failed to open config: {e}"))?
    } else if cmd.default_cfg {
        Config::default()
    } else {
        let (conf, _) =
            Config::find().map_err(|e| format!("Failed to find and open config: {e}"))?;
        conf
    };

    // Do the formatting
    if cmd.all {
        let cur_dir = env::current_dir()
            .map_err(|e| format!("Could not get current working directory: {e}"))?;
        let mut root = files::revtraverse(cur_dir, ".wwwfmt.toml")
            .map_err(|e| format!("Could not get project's root directory: {e}"))?;
        root.pop();
        fmt::all(root, &cfg, minify, inplace)
    } else if let Some(path) = cmd.file {
        fmt::file(path, None, &cfg, minify, inplace)
            .map_err(|e| format!("Failed to format file: {e}"))
    } else {
        Err("You must specify what you want to format (either --all or a --file).".into())
    }
}
