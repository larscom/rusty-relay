use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(long, short, env = "RUSTY_RELAY_SERVER")]
    /// The rusty-relay-server hostname e.g: localhost:8080 or my.server.com
    pub server: String,

    #[arg(long, env = "RUSTY_RELAY_TOKEN")]
    /// The connection token generated on rusty-relay-server
    pub token: String,

    #[arg(long, env = "RUSTY_RELAY_TARGET")]
    /// Target URL to local webserver e.g: http://localhost:3000/api/webhook
    pub target: String,

    #[arg(long, short)]
    /// Connect to rusty-relay-server without TLS
    pub insecure: bool,

    #[arg(long, short, env = "RUSTY_RELAY_CA_CERT")]
    /// Path to CA certificate (PEM encoded)
    pub ca_cert: Option<String>,
}

pub fn args() -> Args {
    Args::parse()
}
