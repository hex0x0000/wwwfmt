use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use oxc::allocator::Allocator;

use crate::{config::Config, fmt};

fn should_ignore(path: &PathBuf, config: &Config) -> bool {
    if let Some(path) = path.to_str() {
        config.ignore_path.contains(&path.to_owned())
    } else {
        false
    }
}

#[inline]
pub fn get_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_lowercase())
}

pub fn recurse_dir(
    path: &PathBuf,
    root: &Option<PathBuf>,
    config: &Config,
    minify: bool,
    inplace: bool,
    alloc: &Allocator,
) -> Result<(), String> {
    for entry in fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory {}: {e}", path.display()))?
    {
        match entry {
            Ok(f) if f.path().is_file() && !should_ignore(&f.path(), config) => {
                let ext = match get_extension(&f.path()) {
                    Some(e) => e,
                    None => continue,
                };
                fmt::inner_file(&f.path(), root, ext, config, minify, inplace, alloc)
                    .map_err(|e| format!("{}: {e}", f.path().display()))?
            }
            Ok(d) if d.path().is_dir() && !should_ignore(&d.path(), config) => {
                recurse_dir(&d.path(), root, config, minify, inplace, alloc)?
            }
            _ => continue,
        }
    }
    Ok(())
}

/// Reverse traversal to find a certain file from a filename
pub fn revtraverse(path: PathBuf, find: &str) -> io::Result<PathBuf> {
    if !path.is_dir() || !path.has_root() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Path must be a full directory.".to_string(),
        ));
    }
    let mut curpath = Some(path);
    while let Some(path) = curpath {
        for entry in fs::read_dir(&path)? {
            let path = entry?.path();
            if path.is_file() && path.file_name().map(|f| f == find).unwrap_or(false) {
                return Ok(path);
            }
        }
        curpath = path.parent().map(|p| p.to_path_buf());
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Reached root directory: {find} not found."),
    ))
}

/// Gets current directory (shorthand)
#[inline(always)]
pub fn get_currdir() -> Result<PathBuf, String> {
    env::current_dir().map_err(|e| format!("Failed to get current working directory: {e}"))
}

/// Returns new path for the formatted file
pub fn outdir(
    mut file_path: PathBuf,
    config: &Config,
    root: &Option<PathBuf>,
    minify: bool,
) -> Result<PathBuf, String> {
    if let Some(root) = root {
        if file_path.starts_with(&root) {
            let root_path: Vec<&OsStr> = root.iter().collect();
            let mut file_path: Vec<&OsStr> = file_path.iter().collect();
            let outdir = if minify {
                config.uglify_outdir.clone()
            } else {
                config.prettify_outdir.clone()
            }
            .ok_or("No outdir specified")?;
            file_path.insert(root_path.len(), OsStr::new(&outdir));

            let mut new_filepath = PathBuf::new();
            for dir in file_path {
                new_filepath.push(dir);
            }
            Ok(new_filepath)
        } else {
            Err(format!(
                "{} not in project's directory",
                file_path.display()
            ))
        }
    } else {
        let ext = file_path
            .extension()
            .and_then(|x| x.to_str())
            .ok_or("Invalid file extension")?;
        if minify {
            file_path.set_extension(format!("min.{ext}"));
        } else {
            file_path.set_extension(format!("prty.{ext}"));
        }
        Ok(file_path)
    }
}

/// Opens file and reads its content
pub fn read(path: &Path) -> Result<String, String> {
    let mut buf = String::new();
    File::open(path)
        .map_err(|e| format!("Failed to open file to format: {e}"))?
        .read_to_string(&mut buf)
        .map_err(|e| format!("Failed to read file to format: {e}"))?;
    Ok(buf)
}

/// Writes data to file, in the right mode and in the right directory.
/// Creates the output directory if it doesn't exist.
pub fn write(path: &PathBuf, out_path: Option<PathBuf>, data: &[u8]) -> Result<(), String> {
    let mut file = if let Some(mut out_path) = out_path {
        let out_file = out_path.clone();
        out_path.pop();
        fs::create_dir_all(&out_path).map_err(|e| format!("Failed to create dir: {e}"))?;
        File::create(out_file).map_err(|e| format!("Failed to create file: {e}"))?
    } else {
        File::create(path).map_err(|e| format!("Failed to open file to format: {e}"))?
    };
    file.write_all(data)
        .map_err(|e| format!("Failed to write to file: {e}"))?;
    Ok(())
}
