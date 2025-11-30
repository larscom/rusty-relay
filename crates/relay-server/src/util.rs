use axum::http::HeaderMap;
use std::{collections::HashMap, fmt::Display, str::FromStr};

pub fn get_query(params: Vec<(String, String)>) -> String {
    let mut qs = String::new();
    for (i, (k, v)) in params.iter().enumerate() {
        let sep = if i == 0 { "?" } else { "&" };
        qs.push_str(&format!("{sep}{k}={v}"));
    }
    qs
}

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

#[cfg(test)]
mod tests {
    use crate::util::get_query;

    #[test]
    fn test_get_query() {
        let qs = get_query(vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
            ("c".to_string(), "3".to_string()),
            ("c".to_string(), "4".to_string()),
        ]);
        assert_eq!(qs, "?a=1&b=2&c=3&c=4");
    }
}
