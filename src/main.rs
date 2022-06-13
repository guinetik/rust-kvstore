use std::{iter::Skip, env::Args};

static DEBUG:bool = true;

enum Log {
    VERBOSE, //always want to print
    DEBUG //only print if debug
}

fn log(log_level: Log, message: String) {
    match log_level {
        Log::VERBOSE => {
            println!("{}", message);
        },
        Log::DEBUG => {
            if DEBUG {
                println!("{}", message);
            }
        }
    }
}

fn main() {
    // In rust, everything is IMUTABLE by default, so we need to opt-in for mutability by using the mut keyword
    let mut prog_args: Skip<Args> = std::env::args() // the Args type is an iterator.
        .skip(1); // here we use the skip method to skip the first value, which is the program path
    // GETTING ARGUMENTS
    let key_arg:String = prog_args
    .next() // the next argument should be the store key.
    .expect("Key not found"); // In rust, Optional is implemented by default, and objects like iterators return an Optional<T> which needs to be unwrapped before reading
    let value_arg:String = prog_args.next().expect("Value not found");
    // the next argument should be the store key.
    log(Log::DEBUG, format!("{} = {}", key_arg, value_arg));
    // formatting the key-value pair
    let file_content: String= format!("{}\t{}\n", key_arg, value_arg);
    // reading from file system
    // fs module returns a Result<(), Error> where () is known as an empty tuple, or a unit, which is similar to void in other langs.
    let write_result = std::fs::write("kv.db", file_content);
    // using functional-like pattern matching
    match write_result {
        Ok(()) => { // Ok is the result where everything went well. notice the empty tuple returned
            log(Log::VERBOSE, "entry saved".to_string());
        },
        Err(e) => {
            log(Log::VERBOSE, "could not write key-value pair".to_string());
            log(Log::DEBUG, format!("{:?}", e));
        }
    }
}
