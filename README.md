# 🦀 Rusty Relay

<p align="center">
   <img src="https://img.shields.io/crates/v/rusty-relay-server" alt="CI">
  <a href="https://github.com/larscom/rusty-relay/actions/workflows/workflow.yml"><img src="https://github.com/larscom/rusty-relay/actions/workflows/workflow.yml/badge.svg" alt="CI"></a> 
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/crates/l/rusty-relay-server" alt="License"></a>
</p>

<p align="center">
  A self-hostable server that forwards webhooks and can proxy all kinds HTTP requests to a local machine.
</p>

---

<p align="center">
   <img src="https://github.com/larscom/rusty-relay/blob/main/.github/image/client-example.png" alt="client">
</p>

## 🚀 What is Rusty Relay?

Rusty Relay is a simple server that can forward webhooks and can proxy all kinds of HTTP requests to a local machine. So you can for example expose your local REST API or website to the public. It's a simplified version of the popular [ngrok](https://ngrok.com/), but instead focuses on being simple and self hostable and purely made for development.

## ✨ Features

- **Relay webhooks**: Forwards webhooks to a local machine.
- **Proxy HTTP requests**: Proxies HTTP requests to a local machine so you can quickly build a REST api or React app locally and expose it to the public.
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

Or simply download the client binary for your platform: [releases](https://github.com/larscom/rusty-relay/releases)

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
  --target http://localhost:3000 \
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

Or simply download the server binary for your platform: [releases](https://github.com/larscom/rusty-relay/releases)

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

## 📚 Examples

### 🪝 Webhooks

Lets say you have a REST API running locally with this endpoint: `http://localhost:3000/api/webhook` ready to receive webhooks.

You run the client

```bash
rusty-relay-client \
  --server my.server.com \
  --target http://localhost:3000/api/webhook \
  --token my_token
```

You should then be able to publically send webhooks to: `https://my.server.com/webhook/{id}`

### 🌐 Serve locally built REST API

Lets say you have a REST API running locally with this endpoint: `http://localhost:3000/api/users`

You run the client

```bash
rusty-relay-client \
  --server my.server.com \
  --target http://localhost:3000 \
  --token my_token
```

You should then be able to access it publically via: `https://my.server.com/proxy/{id}/api/users`

### ⚛️ Serve locally built React application

If you want to serve your `react` application locally, you first have to build it with `vite` e.g. `npm run build` and then use a simple webserver like `http-server` to serve it.

For example:

```bash
# install http-server globally via npm
npm install -g http-server

# serve your freshly build react app on port 3000
http-server ./dist --port 3000
```

Then run the client

```bash
rusty-relay-client \
  --server my.server.com \
  --target http://localhost:3000 \
  --token my_token
```

You should then be able to access it publically via: `https://my.server.com/proxy/{id}`

## ⚖️ Webhook vs Proxy endpoint

The `/webhook/{id}` endpoint returns a `200` or `400` status code immediately and does NOT await the response of the local webserver. A `400` status code is returned when `{id}` does not exist. Otherwise a `200` is returned.

The `/proxy/{id}` endpoint awaits the response of the local webserver, including its status code, body, headers.

## 🌍 Environment variables

### Server environment variables

| Variable                    | Description                                               | Required | Default                   |
| --------------------------- | --------------------------------------------------------- | -------- | ------------------------- |
| `RUSTY_RELAY_HTTP_PORT`     | HTTP port on which the server will listen                 | ❌       | `8080`                    |
| `RUSTY_RELAY_HTTPS_PORT`    | HTTPS port on which the server will listen                | ❌       | `8443`                    |
| `RUSTY_RELAY_CONNECT_TOKEN` | Make the connection token static                          | ❌       | `<auto generated>`        |
| `RUSTY_RELAY_PROXY_TIMEOUT` | How long to await the proxy response (maximum) in seconds | ❌       | `5`                       |
| `RUSTY_RELAY_PING_INTERVAL` | The interval (in seconds) at which to ping the client     | ❌       | `25`                      |
| `RUSTY_RELAY_TLS_CERT_FILE` | Path to TLS certificate (PEM encoded)                     | ❌       | `./certs/cert.pem`        |
| `RUSTY_RELAY_TLS_KEY_FILE`  | Path to TLS private key                                   | ❌       | `./certs/key.pem`         |
| `RUST_LOG`                  | The log level, set to `debug` to enable debug logs        | ❌       | `rusty_relay_server=info` |

### Client environment variables

If you set the `RUSTY_RELAY_SERVER`, `RUSTY_RELAY_TOKEN`, `RUSTY_RELAY_TARGET` variables you can use the client without arguments.

| Variable              | Description                                                          |
| --------------------- | -------------------------------------------------------------------- |
| `RUSTY_RELAY_SERVER`  | The rusty-relay-server hostname e.g: localhost:8080 or my.server.com |
| `RUSTY_RELAY_TOKEN`   | The connection token generated on rusty-relay-server                 |
| `RUSTY_RELAY_TARGET`  | Target URL to local webserver e.g: http://localhost:3000/api/webhook |
| `RUSTY_RELAY_CA_CERT` | Path to the CA certificate (PEM encoded)                             |

## 📜 License

This project is licensed under the MIT License.
