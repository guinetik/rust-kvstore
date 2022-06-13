# rust-kvstore
A simple key-value store utility written in Rust.
Inspired from a livestream by Ryan Levick (https://www.youtube.com/watch?v=WnWGO-tLtLA)

# Installing
Since I'm on windows, I used chocolatey pkg manager and installed 2 pkgs:

rust compiler and pkg manager:
```
choco install rust
```

rust up server:
```
choco install rustup.install
```

## Compiling

using cargo build tool to build a release executable:
```
cargo build --release
```

## Running the app

Run the app inside cargo:
```
cargo run -- --help
```

Run the app executable directly:
```
target\release\gui-kvstore --help
```

## Usage
```
gui-kvstore KEY VALUE --debug=true|false --f=default|csv|json|short --store=STORE_NAME
```
Saves a VALUE string with a key with name of KEY. Options:
```
--debug=true|false              - toggles debug output
--f=default|csv|json|short      - specifies the format to read
--store=STORE_NAME              - reads/writes value in a specific db store file
```

Example:
```
λ gui-kvstore key_name key_value
```
Outputs:  

```
Saved 'key_name' with value 'key_value'
```

Attempt to read a value for the provided KEY:
```
gui-kvstore KEY
```

Example:
```
λ gui-kvstore key_name
```

Outputs:  
```
key_value
```

You can also specify a different store to save the key-pair on:
Example:
```
λ gui-kvstore key_name key_value_other_store --store=my_store
```
Outputs:  
```
Saved 'key_name' with value 'key_value_other_store'
```

Attempt to read a value for the provided KEY in the provided STORE_NAME:  
```
`gui-kvstore KEY --store=STORE_NAME`
```

Prints all stores created: 
```
λ gui-kvstore --stores
```
Outputs:  
```bash
Store Name: default.db 
Store Name: new_store.db
```

Print all key-pairs saved in the store:
```
gui-kvstore --print --store=new_store
```

Outputs:
```
Printing all stores...
key_name:key_value_other_store
```