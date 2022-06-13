mod db;
mod log;
fn main() {
    /*
    // In rust, everything is IMUTABLE by default, so we need to opt-in for mutability by using the mut keyword
    let mut prog_args: Skip<Args> = std::env::args() // the Args type is an iterator.
        .skip(1); // here we use the skip method to skip the first value, which is the program path
                  //
                  // GETTING ARGUMENTS
    let key_arg: String = prog_args
        .next() // the next argument should be the store key.
        .expect("Key not found"); // In rust, Optional is implemented by default, and objects like iterators return an Optional<T> which needs to be unwrapped before reading
                                  // the next argument should be the value
    let value_arg: String = prog_args.next().unwrap_or_default();
    */
    // creating a new Logger struct
    let mut logger: log::Logger = log::Logger::new();
    // lets create a binding to save our store name
    let default_store_name: String = String::from("default");
    //
    let mut options_arg: Vec<String> = vec![];
    // another way to parse arguments is collecting them into a vector
    // here we use a more imperative approach to fine-tune the argument usage
    // we could use a cargo Crate to abstract this command line usage, but it's beyond the scope of this app
    let arguments: Vec<String> = std::env::args().collect();
    // if we dont pass at least 2 args we invoke help borrowing the logger
    if arguments.len() < 2 {
        display_version(&logger);
        display_help(&logger);
    } else {
        let key_arg = arguments[1].to_string();
        // if we're passing only 2 arguments, this is most likely a read on the default store, so let's invoke that
        if arguments.len() == 2 {
            // but before that, lets see if we are actually passing a --store argument
            if arguments[1].starts_with("--") {
                run(key_arg, "".to_string(), default_store_name, &logger);
            } else {
                read(key_arg, default_store_name, &logger);
            }
        }
        // if we're passing 3 arguments
        else if arguments.len() >= 3 {
            // creating a binding for the value argument and setting it to empty
            let mut value_arg: String = String::from("");
            // iterate over our arguments
            for (pos, e) in arguments.iter().enumerate() {
                // since we know the first 2 arguments, lets skip them
                if pos >= 2 {
                    // if the argument starts with -- we know it's an option, not a value
                    if e.starts_with("--") {
                        options_arg.push(e.to_owned());
                    }
                    // if it's not an option, we know it's a value
                    else {
                        value_arg = e.to_owned();
                    }
                }
            }
            logger.toggle_debug(get_logger_debug_from_options(&options_arg, false));
            // we call the main run function passing the key, value (can be empty, so we'll read the key instead) and the store name obtained from the options
            run(
                key_arg,
                value_arg,
                get_store_from_options(&options_arg, default_store_name),
                &logger,
            )
        }
    }
}

/**
 * Reads the options vector and checks if the --debug=true flag is set
 */
fn get_logger_debug_from_options(options_arg: &Vec<String>, default_value: bool) -> bool {
    let mut debug_flag = default_value;
    let debug_option = "--debug=";
    for option in options_arg.iter() {
        if option.starts_with(debug_option) {
            debug_flag = option
                .split("=") // splitting on the = sign
                .last() // getting the last split
                .unwrap_or("false")
                .eq("true");
        }
    }
    debug_flag
}

/**
 * Reads the options vector and returns the value of --store=STORE
 */
fn get_store_from_options(options_arg: &Vec<String>, default_store_name: String) -> String {
    let mut store_name = default_store_name;
    let store_name_option = "--store=";
    for option in options_arg.iter() {
        if option.starts_with(store_name_option) {
            store_name = option
                .split("=") // splitting on the = sign
                .last() // getting the last split
                .unwrap_or(&store_name) //unwrapping the result setting the default
                .to_string() //cloning it so we can use it out of this scope
        }
    }
    store_name
}

/**
 * Runs the specified command invoking the corresponding function
 */
fn run(key: String, value: String, store_name: String, logger: &log::Logger) {
    match key.as_str() {
        "--help" => display_help(&logger),
        "--stores" => display_stores(logger),
        "--print" => display_store(logger, store_name),
        "--version" => display_version(logger),
        _ => handle_input(key, value, store_name, logger),
    }
}
/**
 * Handles the user input by checking the value parameter.
 * If it's not empty, insert the value, otherwise read the key
 */
fn handle_input(key: String, value: String, store_name: String, logger: &log::Logger) {
    // if value is empty we want to read the value for the key
    if value == "" {
        read(key, store_name, logger);
    } else {
        // if value is not empty, we insert a new key
        insert(key, value, store_name, logger);
    }
}

/**
 * Reads the value for a key in a store
 */
fn read(key: String, store_name: String, logger: &log::Logger) {
    let mut db = create_db(store_name.to_string(), logger.is_debug);
    let value: String = db.read(key.to_string());
    if value != "" {
        logger.display(value);
    } else {
        logger.display(format!(
            "Key not found: '{}' on store: '{}'",
            key, store_name
        ));
    }
}

/**
 * Inserts a new key-pair in the selected store
 */
fn insert(key: String, value: String, store_name: String, logger: &log::Logger) {
    // creating a new db struct with the store name
    let mut db = create_db(store_name, logger.is_debug);
    logger.debug(format!("using store: '{}'", db.name));
    // inserting key-pair into the db
    db.insert(key.to_owned(), value.to_owned()); // here we use to_owned to 'clone' the key and value since our db struct wants an owned string.
    logger.display(format!("Saved '{}' with value '{}'", key, value)); //here we can use the key and value binds again since we used to_owned above
}

/**
 * Displays the different stores (dbs) created with the app
 */
fn display_stores(logger: &log::Logger) {
    let db = create_db("default".to_string(), logger.is_debug);
    db.print_stores();
}
/**
 * Display all key-pairs within a store
 */
fn display_store(logger: &log::Logger, store_name: String) {
    let db = create_db(store_name, logger.is_debug);
    db.print_store();
}
/**
 * Displays current app version
 */
fn display_version(logger: &log::Logger) {
    //showing the version information from Cargo
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    logger.display(format!("gui-kvstore version: {}", VERSION));
}

/**
 * Shortcut function to always create db in the same way
 */
fn create_db(store_name: String, is_debug: bool) -> db::Database {
    // lets create a binding for where we want our db files to be stored
    let store_path = std::env::temp_dir() //getting home directory
    .join(".gui-kvstore") //joining with app directory
    .join("data")//joining the path for data
    .display() //converting to a displayable object
    .to_string(); //that has a to_string method
    // return a new db instance with our store name, a valid path and if we`re debugging
    db::Database::new(store_name, store_path, is_debug).unwrap()
}

/**
 * Display command usage
 */
fn display_help(logger: &log::Logger) {
    logger.display(format!("Usage:"));
    logger.display(format!("gui-kvstore KEY VALUE"));
    logger.display(format!("Saves a VALUE string with a key with name of KEY"));
    logger.display(format!(""));
    logger.display(format!("gui-kvstore KEY --store=STORE_NAME"));
    logger.display(format!(
        "Attempts to read a value for the provided KEY in the provided STORE_NAME"
    ));
    logger.display(format!(""));
    logger.display(format!("gui-kvstore --stores"));
    logger.display(format!("Prints all the stores created"));
    logger.display(format!(""));
    logger.display(format!("gui-kvstore KEY VALUE --store=STORE_NAME"));
    logger.display(format!(
        "Saves a VALUE string with a key with name of KEY in the store STORE_NAME"
    ));
    logger.display(format!(""));
    logger.display(format!("gui-kvstore --print"));
    logger.display(format!("Prints all key-pairs saved in the default store"));
    logger.display(format!(""));
    logger.display(format!("gui-kvstore --print --store=STORE_NAME"));
    logger.display(format!("Prints all key-pairs saved in the STORE_NAME"));
}
