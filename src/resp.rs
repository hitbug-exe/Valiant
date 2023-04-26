use anyhow::{Error, Result};
use bytes::BytesMut;
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};

// The ASCII value of the carriage return character.
const CARRIAGE_RETURN: u8 = '\r' as u8;

// The ASCII value of the newline character.
const NEWLINE: u8 = '\n' as u8;

// The different types of values that can be stored in the key-value store.
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Value {
    Null, // A null value.
    SimpleString(String), // A simple string value.
    Error(String), // An error message.
    BulkString(String), // A bulk string value.
    Array(Vec<Value>), // An array of values.
}

impl Value {
    // Converts the value to a Redis command.
    // Returns a tuple containing the command and its arguments.
    pub fn to_command(&self) -> Result<(String, Vec<Value>)> {
        match self {
            Value::Array(items) => {
                // If the value is an array, return the first element as the command
                // and the rest of the elements as the arguments.
                return Ok((
                    items.first().unwrap().unwrap_bulk(),
                    items.clone().into_iter().skip(1).collect(),
                ));
            }
            _ => Err(Error::msg("not an array")), // Return an error if the value is not an array.
        }
    }

    // Returns the underlying string value of a bulk string value.
    fn unwrap_bulk(&self) -> String {
        match self {
            Value::BulkString(str) => str.clone(), // Return the string value if the value is a bulk string.
            _ => panic!("not a bulk string"), // Panic if the value is not a bulk string.
        }
    }

    // Encodes the value into a Redis protocol-compliant string.
    pub fn encode(self) -> String {
        match &self {
            Value::Null => "$-1\r\n".to_string(), // Null values are represented as "$-1\r\n".
            Value::SimpleString(s) => format!("+{}\r\n", s.as_str()), // Simple string values are represented as "+<string>\r\n".
            Value::Error(msg) => format!("-{}\r\n", msg.as_str()), // Error messages are represented as "-<error message>\r\n".
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s), // Bulk string values are represented as "$<length>\r\n<string>\r\n".
            _ => panic!("value encode not implemented for: {:?}", self), // Panic if the value is not one of the supported types.
        }
    }
}


// Define a RespConnection struct that holds a TcpStream and a buffer of bytes. 
pub struct RespConnection {
    stream: TcpStream,
    buffer: BytesMut,
}

// Implementation of RespConnection methods.
impl RespConnection {
    // Create a new instance of RespConnection.
    // Args:
    // * `stream`: A TcpStream that represents a connection to a remote host.
    // Returns:
    // * `Self`: The new instance of the RespConnection struct.
    pub fn new(stream: TcpStream) -> Self {
        // Initialize a new instance of the RespConnection struct and return it.
        return RespConnection {
            stream,
            buffer: BytesMut::with_capacity(512),
        };
    }

    // Read a value from the remote host.
    // Args: None.
    // Returns:
    // * `Result<Option<Value>>`: The result of the operation. Contains either Some(Value) or None.
    pub async fn read_value(&mut self) -> Result<Option<Value>> {
        // Loop until we get a value.
        loop {
            // Read bytes from the remote host into the buffer.
            let bytes_read = self.stream.read_buf(&mut self.buffer).await?;

            // If we didn't read any bytes, return None.
            if bytes_read == 0 {
                return Ok(None);
            }

            // Try to parse the buffer for a value.
            if let Some((value, _)) = parse_message(self.buffer.split())? {
                // If we found a value, return it.
                return Ok(Some(value));
            }
        }
    }

    // Write a value to the remote host.
    // Args:
    // * `value`: The value to write.
    // Returns:
    // * `Result<()>`: The result of the operation.
    pub async fn write_value(&mut self, value: Value) -> Result<()> {
        // Encode the value and write it to the remote host.
        self.stream.write(value.encode().as_bytes()).await?;

        // Return Ok if everything went well.
        Ok(())
    }
}

// Parse a message from a buffer.
// Args:
// * `buffer`: The buffer to parse.
// Returns:
// * `Result<Option<(Value, usize)>>`: The result of the operation. Contains either Some(Value) or None.
fn parse_message(buffer: BytesMut) -> Result<Option<(Value, usize)>> {
    // Match the first byte of the buffer.
    match buffer[0] as char {
        // If it's a `+`, decode a simple string.
        '+' => decode_simple_string(buffer),
        // If it's a `*`, decode an array.
        '*' => decode_array(buffer),
        // If it's a `$`, decode a bulk string.
        '$' => decode_bulk_string(buffer),
        // If it's something else, return an error.
        _ => Err(Error::msg("unrecognised message type")),
    }
}

// Decode a simple string.
// Args:
// * `buffer`: The buffer to decode.
// Returns:
// * `Result<Option<(Value, usize)>>`: The result of the operation. Contains either Some(Value) or None.
fn decode_simple_string(buffer: BytesMut) -> Result<Option<(Value, usize)>> {
    // Try to read until CRLF.
    if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        // If we read something, parse the string.
        let str = parse_string(line)?;

        // Return the string as a simple string value.
        Ok(None)
    }
}

// Takes a buffer of bytes containing an array of values and decodes it into a `Value::Array`.
// Returns `Ok(Some((Value::Array(items), bytes_consumed)))` if successful, where `items` is the
// vector of values contained in the array and `bytes_consumed` is the number of bytes consumed from
// the input buffer. If the input buffer does not contain a complete array, returns `Ok(None)`.
fn decode_array(buffer: BytesMut) -> Result<Option<(Value, usize)>> {
    // Read the length of the array and the number of bytes consumed from the input buffer.
    let (array_length, mut bytes_consumed) =
        if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
            // Parse the length of the array from the input buffer.
            let array_length = parse_integer(line)?;

            (array_length, len + 1)
        } else {
            // If the input buffer does not contain a complete array, return `Ok(None)`.
            return Ok(None);
        };

    // Decode each value in the array and add it to the `items` vector.
    let mut items: Vec<Value> = Vec::new();
    for _ in 0..array_length {
        if let Some((v, len)) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))? {
            items.push(v);
            bytes_consumed += len
        } else {
            // If the input buffer does not contain a complete array, return `Ok(None)`.
            return Ok(None);
        }
    }

    // Return the vector of values contained in the array and the number of bytes consumed from
    // the input buffer as a tuple wrapped in `Ok(Some())`.
    return Ok(Some((Value::Array(items), bytes_consumed)));
}

// Takes a buffer of bytes containing a bulk string and decodes it into a `Value::BulkString`.
// Returns `Ok(Some((Value::BulkString(parse_string(&buffer[bytes_consumed..end_of_bulk])?), end_of_bulk_line)))` 
// if successful, where `parse_string(&buffer[bytes_consumed..end_of_bulk])?` is the string value
// contained in the bulk string, and `end_of_bulk_line` is the index of the next byte after the
// end of the bulk string in the input buffer. If the input buffer does not contain a complete
// bulk string, returns `Ok(None)`.
fn decode_bulk_string(buffer: BytesMut) -> Result<Option<(Value, usize)>> {
    // Read the length of the bulk string and the number of bytes consumed from the input buffer.
    let (bulk_length, bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        // Parse the length of the bulk string from the input buffer.
        let bulk_length = parse_integer(line)?;

        (bulk_length, len + 1)
    } else {
        // If the input buffer does not contain a complete bulk string, return `Ok(None)`.
        return Ok(None);
    };

    // Calculate the index of the last byte in the bulk string and the index of the next byte after
    // the end of the bulk string in the input buffer.
    let end_of_bulk = bytes_consumed + (bulk_length as usize);
    let end_of_bulk_line = end_of_bulk + 2;

    return if end_of_bulk_line <= buffer.len() {
        // If the input buffer contains a complete bulk
        Ok(Some((
            Value::BulkString(parse_string(&buffer[bytes_consumed..end_of_bulk])?),
            end_of_bulk_line,
        )))
    } else {
        Ok(None)
    };
}


// Function: read_until_crlf
//
// Description:
// This function takes a slice of bytes 'buffer' and searches for the first occurrence of
// a carriage return character followed by a newline character. If found, it returns a tuple
// containing a slice of bytes from the start of the buffer to just before the CRLF and the
// index of the byte immediately after the CRLF. If not found, it returns None.
//
// Args:
// - buffer: a slice of bytes to search for CRLF
//
// Returns:
// - Some tuple containing a slice of bytes and an index if CRLF found, None otherwise
//
// Example use:
// let buffer = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
// let (line, index) = read_until_crlf(buffer).unwrap();
// assert_eq!(line, b"GET / HTTP/1.1");
// assert_eq!(index, 18);

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
  for i in 1..buffer.len() {
  if buffer[i - 1] == CARRIAGE_RETURN && buffer[i] == NEWLINE {
    return Some((&buffer[0..(i - 1)], i + 1));
  }
}
  return None;
}

// Function: parse_string
//
// Description:
// This function takes a slice of bytes 'bytes' and attempts to convert it to a valid UTF-8
// encoded string. If successful, it returns a Result containing the string. If unsuccessful,
// it returns a Result containing an Error with the message "Could not parse string".
//
// Args:
// - bytes: a slice of bytes to convert to a string
//
// Returns:
// - Ok containing the string if successful, Err containing an Error otherwise
//
// Example use:
// let bytes = b"hello world";
// let string = parse_string(bytes).unwrap();
// assert_eq!(string, "hello world");

fn parse_string(bytes: &[u8]) -> Result<String> {
String::from_utf8(bytes.to_vec()).map_err(|_| Error::msg("Could not parse string"))
}

// Function: parse_integer
//
// Description:
// This function takes a slice of bytes 'bytes' and attempts to convert it to an integer value.
// It does this by first converting the bytes to a string using parse_string and then attempting
// to parse the string as an integer. If successful, it returns a Result containing the integer.
// If unsuccessful, it returns a Result containing an Error with the message "Could not parse integer".
//
// Args:
// - bytes: a slice of bytes to convert to an integer
//
// Returns:
// - Ok containing the integer if successful, Err containing an Error otherwise
//
// Example use:
// let bytes = b"123";
// let integer = parse_integer(bytes).unwrap();
// assert_eq!(integer, 123);

fn parse_integer(bytes: &[u8]) -> Result<i64> {
let str_integer = parse_string(bytes)?;
(str_integer.parse::<i64>()).map_err(|_| Error::msg("Could not parse integer"))
}