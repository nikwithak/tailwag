use std::{collections::HashMap, io::Read, path::Path};

use tailwag_utils::{files::FileWalker, strings::SanitizeXml, templates::templatize_file};

use super::http::route::{PathVar, Response};

#[derive(Clone)]
pub struct StaticFiles {
    files: HashMap<String, Vec<u8>>,
}

impl StaticFiles {
    pub fn load_static_dir(path: &Path) -> Result<Self, crate::Error> {
        let mut static_files = Self::empty();
        let walker = FileWalker::new(path);
        for (file, path) in walker {
            let mut bytes = Vec::new();
            file?.read_to_end(&mut bytes)?;
            static_files.files.insert(
                // This is a mouthful. Probably an easier way to simplify this?
                path.strip_prefix(&path)?.to_str().ok_or("".to_string())?.to_string(),
                bytes,
            );
        }

        Ok(static_files)
    }

    pub fn empty() -> Self {
        Self {
            files: Default::default(),
        }
    }
}

impl Default for StaticFiles {
    fn default() -> Self {
        Self::load_static_dir(Path::new("static")).unwrap_or(Self::empty())
    }
}

// TODO: Pre-load the templates /statics

pub fn load_template<T: serde::Serialize>(
    filename: &str,
    obj: T,
) -> Response {
    // TODO: This is gross and inefficient.
    let data = serde_json::to_value(obj)
        .unwrap()
        .as_object()
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let Ok(mut filled) = templatize_file(
        // TODO: Pre-load the templates /statics
        Path::new(&format!("static/{}", filename)),
        &data,
    )
    .map(|filled| {
        // TODO: Very inefficient. Need to swap this to do only a single pass through the string.
        filled.sanitize_xml()
    }) else {
        return Response::bad_request();
    };
    let Some(filename) = filename.strip_suffix(".template") else {
        return Response::not_found();
    };
    let mime_type = get_content_type(filename);
    let mime_type = if mime_type == "text/markdown" {
        let mut rendered_html = String::new();
        pulldown_cmark::html::push_html(&mut rendered_html, pulldown_cmark::Parser::new(&filled));
        filled = rendered_html;
        "text/html"
    } else {
        mime_type
    };

    Response::ok()
        .with_body(filled.bytes().collect())
        .with_header("content-type", mime_type)
}

fn get_content_type(filename: &str) -> &'static str {
    filename
        .split('.')
        .last()
        .map(|ext| match ext {
            // TODO: Do I need to enum this out?
            // Probably should at some point.
            "html" => "text/html",
            "css" => "text/css",
            "json" => "application/json",
            "pdf" => "application/pdf",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "png" => "image/png",
            "webp" => "image/webp",
            "md" => "text/markdown",
            _ => "application/octet-stream",
        })
        .unwrap_or("application/octet-stream")
}

pub fn load_static(filename: PathVar<String>) -> Response {
    let filename = filename.0;
    let Ok(mut body) = std::fs::read(format!("static/{}", filename)) else {
        return Response::bad_request();
    };
    // TODO: DRY out to MimeType type
    let mime_type = get_content_type(&filename);

    let mime_type = if mime_type == "text/markdown" {
        let mut rendered_html = String::new();
        pulldown_cmark::html::push_html(
            &mut rendered_html,
            pulldown_cmark::Parser::new(&String::from_utf8(body).unwrap().sanitize_xml()),
            // pulldown_cmark::Parser::new(&String::from_utf8(body).unwrap().sanitize_xml()),
        );
        body = rendered_html.into_bytes();
        "text/html"
    } else {
        mime_type
    };

    Response::ok()
        .with_body(body)
        // TODO: Parse the file extension into a content-type MIME-Type
        .with_header("content-type", mime_type)
}
