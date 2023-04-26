use anyhow::Result;
use resp::Value::{BulkString, Error, Null, SimpleString};
use std::sync::{Arc, Mutex};
use store::Store;
use tokio::net::{TcpListener, TcpStream};

mod resp;
mod store;


/*
Description: This function receives a TcpStream and an Arc<Mutex<Store>> object as arguments and handles a connection to a key-value store.

Args:
- stream: TcpStream object that represents the connection to the client
- client_store: Arc<Mutex<Store>> object that represents the key-value store

Returns:
- Result<()>: An empty Ok result is returned on success, or an Err result on failure.
*/

async fn handle_connection(stream: TcpStream, client_store: Arc<Mutex<Store>>) -> Result<()> {
  // Create a RespConnection object for the stream.
let mut conn = resp::RespConnection::new(stream);

// Loop continuously to handle incoming commands until the connection is closed.
loop {

    // Read the next value from the connection.
    let value = conn.read_value().await?;

    if let Some(value) = value {

        // Convert the value to command and its arguments.
        let (command, args) = value.to_command()?;

        // Match the command with a handler function and return the response.
        let response = match command.to_ascii_lowercase().as_ref() {
            // Handle the "ping" command with a "PONG" response.
            "PING" => SimpleString("PONG".to_string()),

            // Handle the "echo" command with the first argument as the response.
            "ECHO" => args.first().unwrap().clone(),

            // Handle the "get" command by retrieving the value associated with the key from the store.
            "GET" => {
                if let Some(BulkString(key)) = args.get(0) {
                    if let Some(val) = client_store.lock().unwrap().get(key.clone()) {
                        SimpleString(val)
                    } else {
                        Null
                    }
                } else {
                    Error("Get requires one argument".to_string())
                }
            },

            // Handle the "set" command by setting the value associated with the key in the store.
            "SET" => {
                if let (Some(BulkString(key)), Some(BulkString(value))) = (args.get(0), args.get(1)) {
                    client_store.lock().unwrap().set(key.clone(), value.clone());
                    SimpleString("OK".to_string())
                } else {
                    Error("Set requires two arguments".to_string())
                }
            },

            // Handle the "del" command by deleting the key and its associated value from the store.
            "DEL" => {
                if let Some(BulkString(key)) = args.get(0) {
                    if let Some(val) = client_store.lock().unwrap().get(key.clone()) {
                        client_store.lock().unwrap().del(key.clone());
                        SimpleString("DELETED".to_string())
                    } else {
                        Null
                    }
                } else {
                    Error("Del requires one argument".to_string())
                }
            },

            // Handle the "exists" command by checking if the key exists in the store.
            "EXISTS" => {
                if let Some(BulkString(key)) = args.get(0) {
                    if let Some(val) = client_store.lock().unwrap().get(key.clone()) {
                        SimpleString("1".to_string())
                    } else {
                        SimpleString("0".to_string())
                    }
                } else {
                    Error("Exists requires one argument".to_string())
                }
            },

            // If the command is not implemented, return an error response.
            _ => Error(format!("command not implemented: {}", command)),
        };

        // Write the response back to the connection.
        conn.write_value(response).await?;

    } else {

        // If there are no more values to proccess
        break;
    }
  }
  Ok(())
}

/*

Description: This is the main function for a Rust key-value store. It listens for incoming TCP connections on "127.0.0.1:4200" and spawns a new async task to handle each connection. It uses a shared store represented by an Arc wrapped around a Mutex, to handle all incoming requests.
Args: None
Returns: A Result type indicating whether the function executed successfully or an error occurred.
*/
#[tokio::main]
async fn main() -> Result<()> {
  // Bind the TCP listener to "127.0.0.1:4200"
  let listener = TcpListener::bind("127.0.0.1:4200").await?;
  // Create a shared store using an Arc wrapped around a Mutex
  let main_store = Arc::new(Mutex::new(Store::new()));
  
  // Enter an infinite loop to handle incoming connections
  loop {
  // Accept incoming connections
    let incoming = listener.accept().await;
    // Clone the shared store for each incoming connection
    let client_store = main_store.clone();
    // Handle incoming connections in a separate async task
    match incoming {
      
      Ok((stream, _)) => {
      println!("accepted new connection");
      tokio::spawn(async move {
        handle_connection(stream, client_store).await.unwrap();
      });
      }
      Err(e) => {
        println!("error: {}", e);
      }
    }
  }
}