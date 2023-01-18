# navajo

An encrypted chat sevice written by Rust. Also there is a Vert.x version: https://github.com/xiaoyuz/avars

## Building
To launch DBs:

```bash
make navajo-server-deps
```

To build:

```bash
cargo build
```

To start 2 clients as test nodes:

```bash
target/debug/client configs/client-test-1.toml
target/debug/client configs/client-test-2.toml
```

To start server:

```bash
target/debug/server
```

