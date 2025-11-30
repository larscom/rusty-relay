use crate::util;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use tracing::debug;

pub async fn catch_all_handler(
    Path(path): Path<String>,
    Query(params): Query<Vec<(String, String)>>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(client_id) = jar.get("client_id") {
        let client_id = client_id.value();
        let path = format!("/proxy/{client_id}/{path}");
        let cleaned_path = path.replace("//", "/");

        let qs = util::get_query(params);
        let url = format!("{cleaned_path}{qs}");

        debug!("client_id: {client_id} incoming path: {path} redirecting to: {url}");
        Redirect::temporary(url.as_str()).into_response()
    } else {
        StatusCode::OK.into_response()
    }
}

pub async fn root_handler(
    Query(params): Query<Vec<(String, String)>>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(client_id) = jar.get("client_id") {
        let client_id = client_id.value();
        let path = format!("/proxy/{client_id}");

        let qs = util::get_query(params);
        let url = format!("{path}{qs}");

        debug!("client_id: {client_id} incoming path: / redirecting to: {url}");
        Redirect::temporary(url.as_str()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
