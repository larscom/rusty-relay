use std::ffi::OsStr;

const VERSION: &str = "{{version}}";

pub fn print_version() -> bool {
    if std::env::args_os().any(is_version_flag) {
        println!("rusty-relay-client {VERSION}");
        true
    } else {
        false
    }
}

fn is_version_flag<S: AsRef<OsStr>>(s: S) -> bool {
    let s = s.as_ref().to_string_lossy();
    s == "-v" || s == "--version"
}
