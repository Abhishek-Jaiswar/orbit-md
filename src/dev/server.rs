//! Minimal static file server for the Orbit dev command.

use std::fs;
use std::io::Cursor;
use std::net::TcpListener;
use std::path::{Path, PathBuf};

use tiny_http::{Header, Method, Response, Server, StatusCode};

use crate::error::OrbitError;

/// Serves `root` at `host:port` until the process exits.
pub fn serve(root: &Path, host: &str, port: u16) -> Result<(), OrbitError> {
    let root = root.canonicalize().map_err(|source| OrbitError::Io {
        path: root.to_path_buf(),
        source,
    })?;

    let addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&addr).map_err(|source| OrbitError::Io {
        path: PathBuf::from(&addr),
        source,
    })?;
    listener
        .set_nonblocking(false)
        .map_err(|source| OrbitError::Io {
            path: PathBuf::from(&addr),
            source,
        })?;

    let server = Server::from_listener(listener, None).map_err(|err| {
        OrbitError::Config(format!("failed to start dev server on {addr}: {err}"))
    })?;

    for request in server.incoming_requests() {
        let response = handle_request(&root, request.method(), request.url());
        let _ = request.respond(response);
    }

    Ok(())
}

fn handle_request(root: &Path, method: &Method, url: &str) -> Response<Cursor<Vec<u8>>> {
    if method != &Method::Get {
        return text_response(StatusCode(405), "Method Not Allowed");
    }

    match resolve_file(root, url) {
        Ok((bytes, content_type)) => {
            let mut response = Response::from_data(bytes);
            response.add_header(Header::from_bytes("Content-Type", content_type).unwrap());
            response.add_header(Header::from_bytes("Cache-Control", "no-cache").unwrap());
            response
        }
        Err(status) => text_response(StatusCode(status), "Not Found"),
    }
}

fn resolve_file(root: &Path, url: &str) -> Result<(Vec<u8>, &'static str), u16> {
    let mut path = url.split('?').next().unwrap_or("/").trim_start_matches('/');

    if path.is_empty() || path.ends_with('/') {
        path = "index.html";
    }

    let candidate = root.join(path);
    let file = candidate.canonicalize().map_err(|_| 404u16)?;

    if !file.starts_with(root) {
        return Err(404);
    }

    if file.is_dir() {
        let index = file.join("index.html");
        if index.exists() {
            let bytes = fs::read(&index).map_err(|_| 404u16)?;
            return Ok((bytes, "text/html; charset=utf-8"));
        }
        return Err(404);
    }

    let bytes = fs::read(&file).map_err(|_| 404u16)?;
    Ok((bytes, content_type_for(&file)))
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        _ => "application/octet-stream",
    }
}

fn text_response(status: StatusCode, body: &str) -> Response<Cursor<Vec<u8>>> {
    Response::from_string(body.to_owned()).with_status_code(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_index_for_root_url() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "<html>ok</html>").unwrap();
        let root = dir.path().canonicalize().unwrap();

        let (bytes, content_type) = resolve_file(&root, "/").unwrap();
        assert_eq!(bytes, b"<html>ok</html>");
        assert_eq!(content_type, "text/html; charset=utf-8");
    }

    #[test]
    fn rejects_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "ok").unwrap();
        let root = dir.path().canonicalize().unwrap();

        assert!(resolve_file(&root, "/../secret").is_err());
    }
}
