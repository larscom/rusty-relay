use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "rusty-relay")]
pub struct Args {
    #[arg(long)]
    /// The rusty-relay-server hostname e.g: localhost:8080 or my.server.com
    pub server: String,

    #[arg(long)]
    /// The connection token generated on rusty-relay-server
    pub token: String,

    #[arg(long)]
    /// Target URL to local webserver e.g: http://localhost:3000/api/webhook
    pub target: String,

    #[arg(long)]
    /// Connect to rusty-relay-server without TLS
    pub insecure: bool,

    #[arg(long)]
    /// Path to CA certificate (PEM encoded)
    pub ca_cert: Option<String>,
}

pub fn args() -> Args {
    Args::parse()
}
