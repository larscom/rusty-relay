use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;

pub async fn catch_all_handler(Path(path): Path<String>, jar: CookieJar) -> impl IntoResponse {
    if let Some(client_id) = jar.get("client_id") {
        let client_id = client_id.value();
        let path = format!("/proxy/{client_id}/{path}");
        let cleaned_path = path.replace("//", "/");
        tracing::debug!(
            "catch all client_id: {client_id} incoming path: {path} redirecting to: {cleaned_path}"
        );
        Redirect::temporary(cleaned_path.as_str()).into_response()
    } else {
        StatusCode::OK.into_response()
    }
}
