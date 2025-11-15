use axum::http::HeaderMap;
use std::{collections::HashMap, fmt::Display, str::FromStr};

pub fn from_env_or_else<T, F>(key: &str, f: F) -> T
where
    T: FromStr + Display,
    F: FnOnce() -> T,
{
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(f)
}

pub fn generate_id(length: usize) -> String {
    nanoid::nanoid!(length, &nanoid::alphabet::SAFE[2..])
}

pub fn into_hashmap(headers: HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|(k, v)| {
            v.to_str()
                .ok()
                .map(|v| v.to_string())
                .map(|v| (k.to_string(), v))
        })
        .collect()
}
