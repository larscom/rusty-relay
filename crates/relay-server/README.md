# 🦀 Rusty Relay Server

<p align="center">
  <strong>The core relay server for the Rusty Relay project.</strong>
</p>

---

## 🚀 Overview

`relay-server` is the heart of the Rusty Relay system. It's a high-performance websocket relay server built with Rust, Tokio, and Axum. It is responsible for accepting client connections, managing state, and relaying messages between them.

## ✨ Features

- **Websocket Handling**: Manages websocket connections for real-time communication.
- **State Management**: Keeps track of connected clients and their subscriptions.
- **Message Relaying**: Efficiently relays messages between clients.
- **Health Checks**: Includes a health check endpoint for monitoring.
- **Secure**: Supports TLS for encrypted communication.

## 🏁 Getting Started

To run the relay server, you can use the following command from the root of the workspace:

```bash
cargo run --release -p relay-server
```

### Configuration

The server can be configured using command-line arguments. To see the available options, run:

```bash
cargo run --release -p relay-server -- --help
```

## 📦 Crate

This crate is part of the [Rusty Relay](https://github.com/larscom/rusty-relay) workspace.
