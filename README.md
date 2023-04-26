# Valiant

Valiant is a powerful and efficient in-memory key value store built using Rust, designed to handle incoming commands and store data in a simple and efficient manner. With support for basic operations like setting, retrieving, and deleting key-value pairs, it offers an easy-to-use API and uses the RESP protocol for communication. Valiant has been designed to provide a reliable and secure solution for applications that require fast and lightweight storage.

## Dependencies

Valiant relies on the following Rust crates:

- anyhow = "1.0.70"
- bytes = "1.4.0"
- tokio = {version = "1.27.0", features = ["full"]}

`anyhow` provides flexible and composable error handling, `bytes` provides efficient byte buffer manipulation, and `tokio` provides an asynchronous runtime for building reliable and performant network applications. 

These dependencies are automatically managed by Cargo, Rust's package manager, and will be fetched and compiled on demand when building or running Valiant. 

## Usage

The key value store listens on a TCP port and can be interacted with using Redis' RESP commands. To start the server, run the following command:

  `$ cargo run`

This will start the server listening on port 4200.

## Commands

The following commands are currently implemented:

* `PING` : returns "PONG"
* `ECHO <message>` : returns <message>
* `GET <key>` : retrieves the value associated with the given key
* `SET <key> <value>` : sets the value associated with the given key
* `DEL <key>` : deletes the key and its associated value from the store
* `EXISTS <key>` : checks if the key exists in the store

## Architecture

The key value store is designed to be simple and lightweight. Incoming connections are handled by the `handle_connection` function. The function reads commands from the client, matches them with the appropriate handler function, and returns a response.

The `Store` struct is used to store key-value pairs. It uses a `HashMap` to store the pairs and provides methods to get, set, and delete values by key.

The key value store is designed to be thread-safe. The `Store` struct is wrapped in an `Arc<Mutex<Store>>`, allowing multiple threads to access it safely.

## License

*License?* Valiant is like a rebellious teenager, it refuses to be bound by rules and regulations. However, we understand that the real world requires some sort of legal framework. So, here's the deal: you can use Valiant for whatever purposes you want, but don't come crying to me if something goes wrong. In other words, use it at your own risk. And hey, if you make millions using Valiant, just remember to send me a postcard from your private island. Deal? Deal.

