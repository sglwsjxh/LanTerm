use axum::body::Body;
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "frontend/dist/"]
struct Asset;

pub async fn serve_embedded(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Asset::get(path) {
        Some(file) => file_response(path, file.data.as_ref()),
        None => match Asset::get("index.html") {
            Some(f) => file_response("index.html", f.data.as_ref()),
            None => (StatusCode::NOT_FOUND, "Not Found").into_response(),
        },
    }
}

fn file_response(path: &str, data: &[u8]) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(data.to_vec()))
        .expect("response builder for static asset")
}
