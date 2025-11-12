# 🦀 Rusty Relay

<p align="center">
   <img src="https://img.shields.io/crates/v/rusty-relay-server" alt="CI">
  <a href="https://github.com/larscom/rusty-relay/actions/workflows/workflow.yml"><img src="https://github.com/larscom/rusty-relay/actions/workflows/workflow.yml/badge.svg" alt="CI"></a> 
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/crates/l/rusty-relay-server" alt="License"></a>
</p>

<p align="center">
  A self hostable server that can relay webhooks and other requests to a local webserver using the included CLI client.
</p>

---

## 🚀 What is Rusty Relay?

Rusty Relay is a simple relay server that can forward webhooks and other requests to a local machine. It's a simplified version of the popular [ngrok](https://ngrok.com/), but instead focuses on being simple and self hostable and purely made for development.

## ✨ Features

- **Relay webhooks**: Forwards webhooks to a local machine.
- **Relay frontend**: Serve your newly build **React** application locally and give a public URL to anyone in the world.
- **Secure**: Supports TLS for encrypted communication between server and client.

## 🏁 Getting Started

### Running the Server (docker, preferred way)

The preferred way of running the server is via docker and behind a reverse proxy that handles TLS certificates so the client can connect securely via websockets.

```bash
# a connection token is generated on startup which the client needs to connect (see logs)

docker run ghcr.io/larscom/rusty-relay:latest
```

### Installing the Server (alternative method)

Using cargo
```bash
cargo install rusty-relay-server
```

Or simply download the server binary for your platform: https://github.com/larscom/rusty-relay/releases

### Running the Server (alternative method)

Just run the binary.

### Running the Server in HTTPS mode

By default, the server starts in `HTTP` mode only.

If you want to run the server in `HTTPS` mode instead you need to provide the server certificate and private key. If the server detects those files it will automatically run in `HTTPS` mode.

With docker:
```bash
docker run \
  -v ./cert.pem:/app/certs/cert.pem \
  -v ./key.pem:/app/certs/key.pem \
  ghcr.io/larscom/rusty-relay:latest
```

Without docker:

Create a `certs` folder next to the binary with 2 files:
- cert.pem
- key.pem

Or change the environment variables where the server should look for those files.

### Installing the Client

Using cargo
```bash
cargo install rusty-relay-client
```

Or simply download the client binary for your platform: https://github.com/larscom/rusty-relay/releases

### Running the Client

```bash
Usage: rusty-relay-client [OPTIONS] --server <SERVER> --token <TOKEN> --target <TARGET>

Options:
  -s, --server <SERVER>    The rusty-relay-server hostname e.g: localhost:8080 or my.server.com
      --token <TOKEN>      The connection token generated on rusty-relay-server
      --target <TARGET>    Target URL to local webserver e.g: http://localhost:3000/api/webhook
  -i, --insecure           Connect to rusty-relay-server without TLS
  -c, --ca-cert <CA_CERT>  Path to CA certificate (PEM encoded)
  -h, --help               Print help
```

## 🌍 Environment variables

These are environment variables for the server.

| Variable | Type | Description | Required | Default |
|-----------|------|-------------|-----------|----------|
| `HTTP_PORT` | `int` | HTTP port on which the server will listen | ❌ | `8080` |
| `HTTPS_PORT` | `int` | HTTPS port on which the server will listen | ❌ | `8443` |
| `CONNECT_TOKEN` | `string` | Make the connection token static | ❌ | `<auto generated>` |
| `TLS_CERT_FILE` | `string` | Path to TLS certificate (PEM encoded)  | ❌ | `./certs/cert.pem` |
| `TLS_KEY_FILE` | `string` | Path to TLS private key | ❌ | `./certs/key.pem` |
| `RUST_LOG` | `string` | The log level, set to `debug` to enable debug | ❌ | `rusty_relay_server=info` |

## 📜 License

This project is licensed under the MIT License.
