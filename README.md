# wwwfmt

wwwfmt is a simple formatting tool for webfiles (HTML/JS/CSS), which can be also used as a Rust library.

## CLI Usage

wwwfmt can be used to format entire web projects. Simply generate a configuration file (`.wwwfmt.toml`) on the 
root directory of the project with `wwwfmt --write-default`.

Usage:

```ignore
Usage: wwwfmt [-u] [-p] [-c <cfg>] [--write-default] [-d] [-a] [-f <file>] [--inplace] [--no-inplace]

Format HTML/CSS/JS files

Options:
  -u, --uglify      minify files
  -p, --prettify    prettify files
  -c, --cfg         specify config file (by default wwwfmt tries to find it)
  --write-default   write default configuration
  -d, --default-cfg use default configuration
  -a, --all         minifies all files starting from the root of the project
                    (where the .wwwfmt.toml file lies) into the wwwmin directoy
  -f, --file        minifies a single file
  --inplace         minifies and replaces the file(s) in place
  --no-inplace      prettifies file(s) in new file(s)
  -h, --help        display usage information
```

## Library Usage

Example:

```no_run
use wwwfmt::conf;
wwwfmt::all("/path/to/myfilestominify", &conf::Config::default(), true, false).unwrap();
wwwfmt::all("/path/to/myfilestominifyinplace", &conf::Config::default(), true, true).unwrap();
wwwfmt::file("/path/to/myproject/myfiletoprettify.html", Some("/path/to/myproject"), &conf::Config::default(), false, true).unwrap();
```

## Libraries used

- [lightningcss](https://github.com/parcel-bundler/lightningcss) to format CSS files.
- [oxc](https://github.com/oxc-project/oxc) to format JavaScript (and TypeScript) files.
- [quick-xml](https://github.com/tafia/quick-xml) to parse HTML files (formatting is done by this library).
- [argh](https://github.com/google/argh) for CLI arguments parsing
- [toml](https://github.com/toml-rs/toml) for the configuration

