# 🦀 Rusty Relay

<p align="center">
   <img src="https://img.shields.io/crates/v/rusty-relay-server" alt="CI">
  <a href="https://github.com/larscom/rusty-relay/actions/workflows/workflow.yml"><img src="https://github.com/larscom/rusty-relay/actions/workflows/workflow.yml/badge.svg" alt="CI"></a> 
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/crates/l/rusty-relay-server" alt="License"></a>
</p>

<p align="center">
  A self-hostable server that forwards webhooks and websites to a local web server using the included CLI client.
</p>

---

<p align="center">
   <img src="https://github.com/larscom/rusty-relay/blob/main/.github/image/client-example.png" alt="client">
</p>


## 🚀 What is Rusty Relay?

Rusty Relay is a simple relay server that can forward webhooks and websites to a local machine. It's a simplified version of the popular [ngrok](https://ngrok.com/), but instead focuses on being simple and self hostable and purely made for development.

## ✨ Features

- **Relay webhooks**: Forwards webhooks to a local machine.
- **Relay website**: Serve your newly build **React** application locally and give a public URL to anyone in the world.
- **Secure**: Supports TLS for encrypted communication between server and client.
- **No account setup**: Clients do not need accounts to connect to the server.
- **Low memory usage**: The server (in docker) only uses like 4MB memory.

## 🏁 Getting Started

### Installing the Client

#### 🍺 Homebrew
```bash
# add tap
brew tap larscom/tap

# install binary
brew install larscom/tap/rusty_relay_client
```

#### 📟 Shell
```bash
curl -fsSL https://github.com/larscom/rusty-relay/tree/main/scripts/install.sh | sh
```

##### 📦 Cargo
```bash
cargo install rusty-relay-client
```

Or simply download the client binary for your platform: https://github.com/larscom/rusty-relay/releases

### Running the Client

```bash
Usage: rusty-relay-client [OPTIONS] --server <SERVER> --token <TOKEN> --target <TARGET>

Options:
  -s, --server <SERVER>    The rusty-relay-server hostname e.g: localhost:8080 or my.server.com [env: RUSTY_RELAY_SERVER=]
      --token <TOKEN>      The connection token generated on rusty-relay-server [env: RUSTY_RELAY_TOKEN=]
      --target <TARGET>    Target URL to local webserver e.g: http://localhost:3000/api/webhook [env: RUSTY_RELAY_TARGET=]
  -i, --insecure           Connect to rusty-relay-server without TLS
  -c, --ca-cert <CA_CERT>  Path to CA certificate (PEM encoded) [env: RUSTY_RELAY_CA_CERT=]
  -v, --version            Show version info
  -h, --help               Print help
```

### Running the Client against the test server
You can connect to the test server [rusty-relay.larscom.nl](https://rusty-relay.larscom.nl/health) to see how it works, feel free to use it as you like.

```bash
rusty-relay-client \
  --server rusty-relay.larscom.nl \
  --target http://localhost:8080 \
  --token pSyyI54kOhq8yZcV7YOEMKFw
```

### Running the Server (docker)

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

## ⚛️ Serve React app locally
If you want to serve your `react` application locally, you first have to build it with `vite` e.g. `npm run build` and then use a simple webserver like `http-server` to serve it.

For example:
```bash
# install http-server globally via npm
npm install -g http-server

# serve your freshly build react app on port 8080
http-server ./dist --port 8080
```
Then run the client
```bash
rusty-relay-client \
  --server my.server.com \
  --target http://localhost:8080 \
  --token my_token
```

## 🌍 Environment variables

### Server environment variables

| Variable        | Description                                        | Required | Default                   |
| --------------- | -------------------------------------------------- | -------- | ------------------------- |
| `RUSTY_RELAY_HTTP_PORT`     | HTTP port on which the server will listen          | ❌        | `8080`                    |
| `RUSTY_RELAY_HTTPS_PORT`    | HTTPS port on which the server will listen         | ❌        | `8443`                    |
| `RUSTY_RELAY_CONNECT_TOKEN` | Make the connection token static                   | ❌        | `<auto generated>`        |
| `RUSTY_RELAY_TLS_CERT_FILE` | Path to TLS certificate (PEM encoded)              | ❌        | `./certs/cert.pem`        |
| `RUSTY_RELAY_TLS_KEY_FILE`  | Path to TLS private key                            | ❌        | `./certs/key.pem`         |
| `RUST_LOG`      | The log level, set to `debug` to enable debug logs | ❌        | `rusty_relay_server=info` |

### Client environment variables

If you set the `RUSTY_RELAY_SERVER`, `RUSTY_RELAY_TOKEN`, `RUSTY_RELAY_TARGET` variables you can use the client without arguments.

| Variable              | Description                                             |
| --------------------- | ------------------------------------------------------- |
| `RUSTY_RELAY_SERVER`  | The rusty-relay-server hostname e.g: localhost:8080 or my.server.com |
| `RUSTY_RELAY_TOKEN`   | The connection token generated on rusty-relay-server          |
| `RUSTY_RELAY_TARGET`  | Target URL to local webserver e.g: http://localhost:3000/api/webhook               |
| `RUSTY_RELAY_CA_CERT` | Path to the CA certificate (PEM encoded)                |

## 📜 License

This project is licensed under the MIT License.
