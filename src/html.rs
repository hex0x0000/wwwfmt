use std::{
    io::{BufRead, Cursor, Read, Write},
    path::PathBuf,
};

use oxc::{allocator::Allocator, span::SourceType};
use quick_xml::{
    events::{attributes::Attribute, BytesEnd, BytesStart, Event},
    Reader, Writer,
};

use crate::{config::Config, css, files, javascript};

const EMPTY_TAGS: [&[u8]; 14] = [
    b"area", b"base", b"br", b"col", b"embed", b"hr", b"img", b"input", b"link", b"meta", b"param",
    b"source", b"track", b"wbr",
];

fn position(src: &str, pos: u64) -> String {
    let src = match src.get(0..(pos as usize)) {
        Some(s) => s,
        None => return "unknown position".into(),
    };
    let mut lines = 0;
    let mut cols = 0;
    for c in src.chars() {
        match c {
            '\n' => {
                cols = 0;
                lines += 1;
            }
            _ => cols += 1,
        }
    }
    format!("line {lines}, col {cols}")
}

fn get_attr(tag: &BytesStart, attr: &str) -> Option<Vec<u8>> {
    tag.try_get_attribute(attr)
        .ok()
        .flatten()
        .map(|a| a.value.into_owned())
}

fn move_attrs(attrs: BytesStart) -> Result<BytesStart, String> {
    let mut new = BytesStart::new(String::from_utf8(attrs.name().as_ref().to_vec()).unwrap());
    let attrs: Option<Vec<Attribute>> = attrs.html_attributes().map(|a| a.ok()).collect();
    if let Some(attrs) = attrs {
        new.extend_attributes(attrs);
        Ok(new)
    } else {
        Err("Invalid attributes in this element".into())
    }
}

fn trim_pos<I>(src: I) -> Option<usize>
where
    I: Iterator<Item = char>,
{
    let mut trim = None;
    for c in src {
        match c {
            '\t' | '\n' | ' ' => {
                trim.replace(trim.unwrap_or(0) + 1);
            }
            _ => break,
        }
    }
    trim
}

#[derive(Debug)]
enum BufType {
    Script,
    Style,
    Pre,
}

pub struct Html<'a> {
    /// Arena allocator for javascript formatting
    alloc: &'a Allocator,

    /// Text source of the unformatted text
    src: &'a str,

    /// XML reader, reads unformatted html
    reader: Reader<&'a [u8]>,

    /// User configuration
    config: &'a Config,

    /// XML writer
    writer: Writer<Cursor<Vec<u8>>>,

    /// What kind of buffer is being read
    wbuf: Option<BufType>,

    /// Is minifying or not
    minify: bool,

    /// Whether or not sbuf should be formatted (in case the script is not javascript)
    fmt: bool,

    /// Indentation level
    indent: usize,
}

impl<'a> Html<'a> {
    /// Initialize HTML formatter
    /// Since the source itself is an UTF-8 string, bytes read from the reader are considered UTF-8
    pub fn new(src: &'a str, alloc: &'a Allocator, config: &'a Config) -> Self {
        let mut reader = Reader::from_str(src);
        reader.config_mut().check_comments = true;
        reader.config_mut().check_end_names = false;
        reader.config_mut().trim_markup_names_in_closing_tags = true;
        reader.config_mut().trim_text(false);
        Html {
            alloc,
            src,
            reader,
            config,
            writer: Writer::new(Cursor::new(Vec::new())),
            wbuf: None,
            minify: false,
            fmt: false,
            indent: 0,
        }
    }

    /// Writes indented text
    fn write_indented(&mut self, src: &str) -> Result<(), String> {
        if src.is_empty() {
            return Ok(());
        }
        let indent = self
            .config
            .html
            .prettify_indent_kind
            .repeat(self.config.html.prettify_indent_num * self.indent);
        let writer = self.writer.get_mut();
        for l in src.trim().split('\n') {
            writer
                .write_all(indent.as_bytes())
                .map_err(|e| format!("Failed to write text indent: {e}"))?;
            writer
                .write_all(l.as_bytes())
                .map_err(|e| format!("Failed to write text line: {e}"))?;
            writer
                .write_all(b"\n")
                .map_err(|e| format!("Failed to write text newline: {e}"))?;
        }
        Ok(())
    }

    /// Writes trimmed text, while keeping spaces
    fn write_trimmed(&mut self, src: Vec<u8>) -> Result<(), String> {
        let mut txt = String::from_utf8(src).unwrap();
        if let Some(trim_start) = trim_pos(txt.chars()) {
            txt.drain(..trim_start);
            txt.insert(0, ' ');
        }
        if let Some(trim_end) = trim_pos(txt.chars().rev()) {
            txt.drain((txt.len() - trim_end)..);
            txt.push(' ');
        }
        if txt.is_empty() {
            return Ok(());
        }
        if txt.trim().is_empty() {
            self.writer
                .get_mut()
                .write_all(b" ")
                .map_err(|e| format!("Failed to write text: {e}"))
        } else {
            self.writer
                .get_mut()
                .write_all(txt.as_bytes())
                .map_err(|e| format!("Failed to write text: {e}"))
        }
    }

    fn write_text(&mut self, src: &str) -> Result<(), String> {
        self.writer
            .get_mut()
            .write_all(src.as_bytes())
            .map_err(|e| format!("Failed to write text line: {e}"))
    }

    fn write_end(&mut self, name: &str) -> Result<(), String> {
        if !self.minify {
            self.write_indent()?;
        }
        self.writer
            .write_event(Event::End(BytesEnd::new(name)))
            .map_err(|e| format!("Failed to write end tag: {e}"))?;
        if !self.minify {
            self.write_newline()?;
        }
        Ok(())
    }

    /// Writes and event, clones the tags if attributes are to be formatted
    fn write_event(&mut self, event: Event) -> Result<(), String> {
        let event = match event {
            Event::Start(e) if self.config.html.fmt_attrs => Event::Start(move_attrs(e)?),
            Event::Empty(e) if self.config.html.fmt_attrs => Event::Empty(move_attrs(e)?),
            e => e,
        };
        self.writer
            .write_event(event)
            .map_err(|e| format!("Failed to write event: {e}"))
    }

    /// Writes a new line
    fn write_newline(&mut self) -> Result<(), String> {
        self.writer
            .get_mut()
            .write_all(b"\n")
            .map_err(|e| format!("Failed to write newline: {e}"))
    }

    /// Writes an indent
    fn write_indent(&mut self) -> Result<(), String> {
        let v = self
            .config
            .html
            .prettify_indent_kind
            .repeat(self.config.html.prettify_indent_num * self.indent);
        self.writer
            .get_mut()
            .write_all(v.as_bytes())
            .map_err(|e| format!("Failed to write indent: {e}"))
    }

    fn handle_buf(&mut self) -> Result<(), String> {
        const SCRIPT: [u8; 8] = *b"/script>";
        const STYLE: [u8; 7] = *b"/style>";
        const PRE: [u8; 5] = *b"/pre>";
        if let Some(buftype) = self.wbuf.take() {
            let mut reader = self.reader.stream();
            let mut buf: Vec<u8> = Vec::with_capacity(16384);
            loop {
                reader
                    .read_until(b'<', &mut buf)
                    .map_err(|e| format!("Failed to read {:?} buffer: {e}", buftype))?;
                match buftype {
                    BufType::Script => {
                        let mut endtag = [0u8; 8];
                        reader
                            .read_exact(&mut endtag)
                            .map_err(|e| format!("Failed to read </script>: {e}"))?;
                        if endtag == SCRIPT {
                            break;
                        } else {
                            buf.extend_from_slice(&endtag);
                        }
                    }
                    BufType::Style => {
                        let mut endtag = [0u8; 7];
                        reader
                            .read_exact(&mut endtag)
                            .map_err(|e| format!("Failed to read </style>: {e}"))?;
                        if endtag == STYLE {
                            break;
                        } else {
                            buf.extend_from_slice(&endtag);
                        }
                    }
                    BufType::Pre => {
                        let mut endtag = [0u8; 5];
                        reader
                            .read_exact(&mut endtag)
                            .map_err(|e| format!("Failed to read </pre>: {e}"))?;
                        if endtag == PRE {
                            break;
                        } else {
                            buf.extend_from_slice(&endtag);
                        }
                    }
                }
            }
            buf.pop();
            let buf = String::from_utf8(buf).unwrap();
            match buftype {
                BufType::Script => {
                    if self.fmt && !self.minify {
                        let buf = javascript::fmt_str(
                            &buf,
                            SourceType::cjs(),
                            self.alloc,
                            self.config,
                            false,
                        )?;
                        self.write_indented(&buf)?;
                    } else if !self.fmt && !self.minify {
                        self.write_indented(&buf)?;
                    } else if self.fmt && self.minify {
                        let buf = javascript::fmt_str(
                            &buf,
                            SourceType::cjs(),
                            self.alloc,
                            self.config,
                            true,
                        )?;
                        self.write_text(&buf)?;
                    } else if !self.fmt && self.minify {
                        self.write_text(&buf)?;
                    }
                    self.fmt = false;
                    if self.indent > 0 {
                        self.indent -= 1;
                    }
                    self.write_end("script")?;
                }
                BufType::Style => {
                    if self.minify {
                        let buf = css::fmt_str(&buf, self.config, true)?;
                        self.write_indented(&buf)?;
                    } else {
                        let buf = css::fmt_str(&buf, self.config, true)?;
                        self.write_text(&buf)?;
                    }
                    if self.indent > 0 {
                        self.indent -= 1;
                    }
                    self.write_end("style")?;
                }
                BufType::Pre => {
                    self.write_text(&buf)?;
                    if self.indent > 0 {
                        self.indent -= 1;
                    }
                    self.write_end("pre")?;
                }
            }
        }
        Ok(())
    }

    /// Prettifies everything
    fn prettify_inner(&mut self) -> Result<(), String> {
        self.minify = false;
        let mut noindent = 0;
        loop {
            self.handle_buf()?;
            match self.reader.read_event() {
                // Checks for style or script tags
                Ok(Event::Start(e)) if e.name().as_ref() == b"script" => {
                    if get_attr(&e, "src").is_none() {
                        self.wbuf.replace(BufType::Script);
                        self.fmt = get_attr(&e, "type")
                            .map(|a| a == b"text/javascript" || a == b"module")
                            .unwrap_or(false);
                    }
                    self.write_indent()?;
                    self.write_event(Event::Start(e))?;
                    self.write_newline()?;
                    self.indent += 1;
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"style" => {
                    self.wbuf.replace(BufType::Style);
                    self.write_indent()?;
                    self.write_event(Event::Start(e))?;
                    self.write_newline()?;
                    self.indent += 1;
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"pre" => {
                    self.wbuf.replace(BufType::Pre);
                    self.write_indent()?;
                    self.write_event(Event::Start(e))?;
                    self.write_newline()?;
                    self.indent += 1;
                }

                // Exit
                Ok(Event::Eof) => break,

                // NoIndent, events inside of noindent tags are not formatted
                Ok(Event::Start(e)) if noindent > 0 => {
                    let name = e.name().as_ref().to_ascii_lowercase();
                    if !EMPTY_TAGS.contains(&name.as_ref()) {
                        self.indent += 1;
                        noindent += 1;
                    }
                    self.write_event(Event::Start(e))?;
                }
                Ok(Event::End(e)) if noindent > 0 => {
                    self.indent -= 1;
                    noindent -= 1;
                    self.write_event(Event::End(e))?;
                    if noindent == 0 {
                        self.write_newline()?;
                    }
                }
                Ok(Event::Text(e)) if noindent > 0 => self.write_trimmed(e.to_vec())?,
                Ok(e) if noindent > 0 => self.write_event(e)?,

                // Handle normal tags
                Ok(Event::Start(e)) => {
                    self.write_indent()?;
                    let name = e.name().as_ref().to_ascii_lowercase();
                    if !EMPTY_TAGS.contains(&name.as_ref()) {
                        self.indent += 1;
                    }
                    if self
                        .config
                        .html
                        .prettify_noindent_tags
                        .iter()
                        .any(|t| t.as_bytes() == e.name().as_ref().to_ascii_lowercase())
                    {
                        noindent += 1;
                    }
                    self.write_event(Event::Start(e))?;
                    if noindent == 0 {
                        self.write_newline()?;
                    }
                }
                Ok(Event::End(e)) => {
                    if self.indent > 0 {
                        self.indent -= 1;
                    }
                    self.write_indent()?;
                    self.write_event(Event::End(e))?;
                    self.write_newline()?;
                }
                Ok(Event::Text(e)) => {
                    self.write_indented(core::str::from_utf8(&e).unwrap().trim())?;
                }
                Ok(e) => {
                    self.write_indent()?;
                    self.write_event(e)?;
                    self.write_newline()?;
                }
                Err(e) => return Err(format!("Invalid HTML syntax: {:?}", e)),
            }
        }
        Ok(())
    }

    /// Minifies everything
    fn minify_inner(&mut self) -> Result<(), String> {
        self.minify = true;
        loop {
            self.handle_buf()?;
            match self.reader.read_event() {
                // Checks for style or script tags
                Ok(Event::Start(e)) if e.name().as_ref() == b"script" => {
                    if get_attr(&e, "src").is_none() {
                        self.wbuf.replace(BufType::Script);
                        self.fmt = get_attr(&e, "type")
                            .map(|a| a == b"text/javascript" || a == b"module")
                            .unwrap_or(false);
                    }
                    self.write_event(Event::Start(e))?
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"style" => {
                    self.wbuf.replace(BufType::Style);
                    self.write_event(Event::Start(e))?
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"pre" => {
                    self.wbuf.replace(BufType::Pre);
                    self.write_event(Event::Start(e))?;
                }

                // Ignore comments
                Ok(Event::Comment(_)) if self.config.html.uglify_rm_comments => continue,

                // Trims new lines and tabs
                Ok(Event::Text(e)) => self.write_trimmed(e.to_vec())?,

                // Exit
                Ok(Event::Eof) => break,

                // Handle everything else
                Ok(e) => self.write_event(e)?,
                Err(e) => return Err(format!("Invalid HTML syntax: {:?}", e)),
            }
        }
        Ok(())
    }

    /// Returns the last position, depending on the last error, the last script/style tag, or the
    /// last buffer position
    fn position(&mut self) -> String {
        if self.reader.error_position() != 0 {
            position(self.src, self.reader.error_position())
        } else {
            position(self.src, self.reader.buffer_position())
        }
    }

    /// Consumes itself and returns prettified text as bytes (guaranteed to be UTF-8)
    pub fn prettify(mut self) -> Result<Vec<u8>, String> {
        self.prettify_inner()
            .map_err(|e| format!("At {}: {e}", self.position()))?;
        Ok(self.writer.into_inner().into_inner())
    }

    /// Consumes itself and returns minified text as bytes (guaranteed to be UTF-8)
    pub fn minify(mut self) -> Result<Vec<u8>, String> {
        self.minify_inner()
            .map_err(|e| format!("At {}: {e}", self.position()))?;
        Ok(self.writer.into_inner().into_inner())
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
    let html = Html::new(&file, alloc, config);
    let fmted = if minify {
        html.minify()?
    } else {
        html.prettify()?
    };
    files::write(path, out_path, &fmted)
}
