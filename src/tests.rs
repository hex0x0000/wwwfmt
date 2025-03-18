use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use include_dir::{include_dir, Dir};
use oxc::allocator::Allocator;
use pretty_assertions::assert_eq;
use testdir::testdir;

use crate::fmt;
use crate::{config::Config, html::Html};

static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

fn get_file(file: &str) -> &'static str {
    let contents = ASSETS.get_file(file).unwrap().contents();
    core::str::from_utf8(contents).unwrap()
}

fn copy_files_to(mut outdir: PathBuf) {
    for file in ASSETS.files() {
        outdir.push(file.path().file_name().unwrap());
        File::create_new(&outdir)
            .unwrap()
            .write_all(file.contents())
            .unwrap();
        outdir.pop();
    }
}

#[test]
fn test_html() {
    let alloc = Allocator::new();
    let config = Config::default();
    let html = Html::new(get_file("example.html"), &alloc, &config);
    let ugly = html
        .minify()
        .unwrap_or_else(|e| panic!("Minify failed: {e}"));
    let html = Html::new(get_file("example.html"), &alloc, &config);
    let pretty = html
        .prettify()
        .unwrap_or_else(|e| panic!("Prettify failed: {e}"));
    assert_eq!(
        core::str::from_utf8(&ugly).unwrap(),
        get_file("example.min.html")
    );
    assert_eq!(
        core::str::from_utf8(&pretty).unwrap(),
        get_file("example.prty.html")
    );
}

#[test]
fn test_tree() {
    let path = testdir!();
    {
        let mut path = path.clone();
        path.push(".wwwfmt.toml");
        File::create_new(path).unwrap();
    }
    copy_files_to(path.clone());
    let config = Config::default();
    fmt::all(path.clone(), &config, true, false).expect("Minify failed");
    fmt::all(path, &config, false, true).expect("Prettify failed");
}
