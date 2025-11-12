# 🦀 Rusty Relay Client

<p align="center">
  <strong>A command-line client for the Rusty Relay server.</strong>
</p>

---

## 🚀 Overview

`relay-client` is a command-line tool for interacting with a `relay-server` instance. It allows you to connect to the server, send messages, and subscribe to message channels.

## ✨ Features

- **Connect to Server**: Establish a websocket connection to a relay server.
- **Send Messages**: Send messages to specific channels.
- **Subscribe to Channels**: Subscribe to channels to receive messages.
- **Webhook Forwarding**: Forward messages from a websocket to a webhook.

## 🏁 Getting Started

To run the relay client, you can use the following command from the root of the workspace:

```bash
cargo run --release -p relay-client
```

### Configuration

The client is configured using command-line arguments. To see the available options, run:

```bash
cargo run --release -p relay-client -- --help
```

## 📦 Crate

This crate is part of the [Rusty Relay](https://github.com/larscom/rusty-relay) workspace.
