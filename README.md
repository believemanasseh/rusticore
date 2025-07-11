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
use rusticore::Server;
use rusticore::Route;

fn main() {
    let mut server = Server::new(String::from("localhost"), 9000, false, None);
    let route = Route::new(String::from("GET"), String::from("/hello"), |req, res| {
        res.send(String::from("Hello, world!"));
    });
    server.add_route(route);
    server.start();
}
```

## Testing

To run tests, use:

```sh
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
