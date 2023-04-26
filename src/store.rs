use std::collections::HashMap;

// Store is a struct that represents a key-value store
pub struct Store {
    data: HashMap<String, String>,
}

impl Store {
    // Creates a new instance of the Store struct
    pub fn new() -> Self {
        Store {
            data: HashMap::new(),
        }
    }

    // Adds a new key-value pair to the store
    //
    // # Arguments
    //
    // * `key` - A string that represents the key
    // * `value` - A string that represents the value
    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    // Retrieves the value associated with the given key
    //
    // # Arguments
    //
    // * `key` - A string that represents the key to lookup
    //
    // # Returns
    //
    // An Option<String> that contains the value associated with the key,
    // or None if the key is not present in the store
    pub fn get(&mut self, key: String) -> Option<String> {
        self.data.get(key.as_str()).cloned()
    }

    // Deletes the key-value pair associated with the given key
    //
    // # Arguments
    //
    // * `key` - A string that represents the key to delete
    pub fn del(&mut self, key: String) {
        self.data.remove(key.as_str());
    }
}