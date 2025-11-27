use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use tracing::debug;

pub async fn catch_all_handler(Path(path): Path<String>, jar: CookieJar) -> impl IntoResponse {
    if let Some(client_id) = jar.get("client_id") {
        let client_id = client_id.value();
        let path = format!("/proxy/{client_id}/{path}");
        let cleaned_path = path.replace("//", "/");
        debug!("client_id: {client_id} incoming path: {path} redirecting to: {cleaned_path}");
        Redirect::temporary(cleaned_path.as_str()).into_response()
    } else {
        StatusCode::OK.into_response()
    }
}

pub async fn root_handler(jar: CookieJar) -> impl IntoResponse {
    if let Some(client_id) = jar.get("client_id") {
        let client_id = client_id.value();
        let path = format!("/proxy/{client_id}");
        debug!("client_id: {client_id} incoming path: / redirecting to: {path}");
        Redirect::temporary(path.as_str()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
