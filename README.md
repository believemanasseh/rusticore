# Rusticore

A minimal, customisable and multithreaded web server written in Rust.

## Features

- Simple API for starting and managing the server
- Configurable logging with [log4rs](https://crates.io/crates/log4rs)
- Supports both file and console logging
- Easily extensible for custom logic and routing

## Installation

### From Source

Clone the repository and build with Cargo:

```sh
git clone https://github.com/believemanasseh/rusticore.git
cd rusticore
cargo build --release
```

The compiled binary will be in `target/release/rusticore`.

### From crates.io

Add to your `Cargo.toml`:

```toml
[dependencies]
rusticore = "0.1.0"
```

Then run:

```sh
cargo build --release
```

## Usage

### As a Binary

After building, run the server:

```sh
./target/release/rusticore
```

### As a Library

Import and use in your Rust project:

```rust
use rusticore::server::Server;

fn main() {
    let mut server = Server("localhost".to_string(), 9000, false, None);
    server.start();
    server.add_route("/hello", |req, res| {
        res.send("Hello, world!");
    });
}
```

## Configuration

Logging is configured via `config/log4rs.yaml`. Edit this file to change log levels, output destinations, and formats.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
