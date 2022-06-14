use std::{collections::HashMap, str::Split};
use tabled::{Table, Tabled};
#[path = "crypto.rs"]
mod crypto;
#[path = "db.rs"]
mod db;
#[path = "log.rs"]
mod log;

static DEFAULT_STORE: &str = "default";

#[derive(Tabled)]
struct KeypairItem {
    key: String,
    value: String,
}

pub struct App {
    logger: log::Logger,
    arguments: Vec<String>,
    options_arg: Vec<String>,
    store_name: String,
    keypair: ([u8; 32], [u8; 32]),
}

impl App {
    pub fn new(arguments: Vec<String>) -> App {
        let logger = log::Logger::new();
        //
        App {
            logger,
            arguments,
            options_arg: vec![],
            store_name: "default".to_string(),
            keypair: crypto::generate_key_pair(),
        }
    }

    pub fn init(mut self) {
        let mut opts: Vec<String> = vec![];
        // iterate over our arguments
        for (_pos, e) in self.arguments.iter().enumerate() {
            if e.starts_with("--") {
                opts.push(e.to_owned());
            }
        }
        //
        self.options_arg = opts;
        let debug_opt = self.get_logger_debug_from_options(false);
        self.store_name = self.get_store_from_options(DEFAULT_STORE.to_string());
        self.logger.toggle_debug(debug_opt);
        //
        let kp: ([u8; 32], [u8; 32]);
        let key_path = App::get_data_dir().join("kvstore.key");
        let key_file_exists = key_path.exists();
        //
        if key_file_exists {
            self.logger.debug("key file already exists".to_string());
            let keys_file_contents: String = std::fs::read_to_string(key_path).unwrap();
            let mut keys_split: Split<&str> = keys_file_contents.split("\t");
            let public = &crypto::decode_hex(keys_split.next().unwrap()).unwrap()[..];
            let private = &crypto::decode_hex(keys_split.next().unwrap()).unwrap()[..];

            self.logger.debug(format!("Private bytes: {:?}", private));
            self.logger.debug(format!("Public bytes: {:?}", public));

            kp = (
                public.try_into().expect("incorrect private length"),
                private.try_into().expect("incorrect public length"),
            );
        } else {
            self.logger
                .debug("key doesn't exist, creating...".to_string());
            kp = crypto::generate_key_pair();
            //
            self.logger.debug(format!("Public bytes: {:?}", &kp.0));
            self.logger.debug(format!("Private bytes: {:?}", &kp.1));
            //
            let public = crypto::encode_hex(&kp.0);
            let private = crypto::encode_hex(&kp.1);
            //
            self.logger.debug(format!("Private Hex: {}", private));
            self.logger.debug(format!("Public Hex: {}", public));
            //
            let keypair_content = format!("{}\t{}", public, private);
            let _ = std::fs::write(key_path, keypair_content);
        }
        self.keypair = kp;
        //
        self.store_name = DEFAULT_STORE.to_string();
        // another way to parse arguments is collecting them into a vector
        // here we use a more imperative approach to fine-tune the argument usage
        // we could use a cargo Crate to abstract this command line usage, but it's beyond the scope of this app
        //
        if self.arguments.len() < 2 {
            // if we dont pass at least 2 args we invoke help
            self.print_version();
            self.print_help();
        } else {
            let key_arg = self.arguments[1].to_string();
            // if we're passing only 2 arguments, this is most likely a read on the default store, so let's invoke that
            if self.arguments.len() == 2 {
                // but before that, lets see if we are actually passing a --store argument
                if self.arguments[1].starts_with("--") {
                    self.run(key_arg, "".to_string());
                } else {
                    self.read(key_arg);
                }
            }
            // if we're passing 3 arguments
            else if self.arguments.len() >= 3 {
                // creating a binding for the value argument and setting it to empty
                let mut value_arg: String = String::from("");
                // iterate over our arguments
                for (pos, e) in self.arguments.iter().enumerate() {
                    // since we know the first 2 arguments, lets skip them
                    if pos >= 2 {
                        // if the argument starts with -- we know it's an option, not a value
                        if !e.starts_with("--") {
                            // if it's not an option, we know it's a value
                            value_arg = e.to_owned();
                        }
                    }
                }
                //
                self.run(key_arg, value_arg);
            }
        }
    }

    fn get_store_from_options(&self, default_store_name: String) -> String {
        let mut store_name = default_store_name;
        let store_name_option = "--store=";
        for option in self.options_arg.iter() {
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
     * Reads the options vector and checks if the --debug=true flag is set
     */
    fn get_logger_debug_from_options(&self, default_value: bool) -> bool {
        let mut debug_flag = default_value;
        let debug_option = "--debug=";
        for option in self.options_arg.iter() {
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
     * Reads the options vector and checks if the --debug=true flag is set
     */
    fn get_formatting_from_options(&self, default_value: String) -> String {
        let mut format_value = default_value;
        let format_option = "--f=";
        for option in self.options_arg.iter() {
            if option.starts_with(format_option) {
                format_value = option
                    .split("=") // splitting on the = sign
                    .last() // getting the last split
                    .unwrap_or(&format_value) //unwrapping the result setting the default
                    .to_string()
            }
        }
        format_value
    }

    /**
     * Runs the specified command invoking the corresponding function
     */
    fn run(self, key: String, value: String) {
        match key.as_str() {
            "--help" => self.print_help(),
            "--stores" => self.print_stores(),
            "--print" => self.print_store(),
            "--version" => self.print_version(),
            _ => self.handle_input(key, value),
        }
    }

    /**
     * Handles the user input by checking the value parameter.
     * If it's not empty, insert the value, otherwise read the key
     */
    fn handle_input(self, key: String, value: String) {
        // if value is empty we want to read the value for the key
        if value == "" {
            self.read(key);
        } else {
            // if value is not empty, we insert a new key
            self.insert(key, value);
        }
    }

    /**
     * Reads the value for a key in a store
     */
    fn read(&self, key: String) {
        let mut db = self.create_db();
        let value: String = db.read(key.to_string());
        let formatting = self.get_formatting_from_options("default".to_string());
        if value != "" {
            self.print_keypair_formatted(
                &key,
                crypto::decrypt_string(&self.keypair.1, value), //decrypting value with private key
                formatting,
            );
        } else {
            self.logger.display(format!(
                "Key not found: '{}' on store: '{}'",
                key, self.store_name
            ));
        }
    }

    /**
     * Inserts a new key-pair in the selected store
     */
    fn insert(&self, key: String, value: String) {
        let mut db = self.create_db();
        self.logger.debug(format!("using store: '{}'", db.name));
        // inserting key-pair into the db
        db.insert(
            key.to_owned(), // here we use to_owned to 'clone' the key and value since our db struct wants an owned string.
            crypto::encrypt_string(&self.keypair.0, value.to_owned()), //encrypting the value
        );
        self.logger
            .display(format!("Saved '{}' with value '{}'", key, value)); //here we can use the key and value binds again since we used to_owned above
    }

    /**
     * Displays the different stores (dbs) created with the app
     */
    fn print_stores(&self) {
        let db = self.create_db();
        db.print_stores();
    }

    /**
     * Display all key-pairs within a store
     */
    fn print_store(&self) {
        let formatting = self.get_formatting_from_options("default".to_string());
        self.logger.display(format!(
            "Displaying Store '{}' with formatting '{}'",
            self.store_name, formatting
        ));
        let db = self.create_db();
        let items = db.get_stores();
        self.print_store_formatted(items, formatting);
    }

    /**
     * Shortcut function to always create db in the same way
     */
    fn create_db(&self) -> db::Database {
        // lets create a binding for where we want our db files to be stored
        let store_path = App::get_data_dir() //getting home directory
            .join("data") //joining the path for data
            .display() //converting to a displayable object
            .to_string(); //that has a to_string method
                          // return a new db instance with our store name, a valid path and if we`re debugging
        db::Database::new(
            self.store_name.to_string(),
            store_path,
            self.logger.is_debug,
        )
        .unwrap()
    }

    fn print_version(&self) {
        //showing the version information from Cargo
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        self.logger
            .display(format!("gui-kvstore version: {}", VERSION));
    }

    /**
     * Display command usage
     */
    fn print_help(&self) {
        let logger = &self.logger;
        logger.display(format!("Usage:"));
        logger.display(format!(
        "\tgui-kvstore KEY VALUE --debug=true|false --f=default|csv|json|short|table --store=STORE_NAME"
        ));
        logger.display(format!("Saves a VALUE string with a key with name of KEY"));
        logger.display(format!("\nOptions:"));
        logger.display(format!(
            "\t--debug=true|false               - toggles debug output"
        ));
        logger.display(format!(
            "\t--f=default|csv|json|short|table - specifies the format to read"
        ));
        logger.display(format!(
            "\t--store=STORE_NAME               - reads/writes value in a specific db store file"
        ));
        //
        logger.display(format!("\nOther Commands:"));
        logger.display(format!("\tgui-kvstore --stores"));
        logger.display(format!("Prints all the stores created"));
        logger.display(format!("\n\tgui-kvstore KEY VALUE --store=STORE_NAME"));
        logger.display(format!(
            "Saves a VALUE string with a key with name of KEY in the store STORE_NAME"
        ));
        logger.display(format!(""));
        logger.display(format!(
            "gui-kvstore --print --debug=true|false --f=default|csv|json|short|table --store=STORE_NAME"
        ));
        logger.display(format!("Prints all key-pairs saved in the store"));
        logger.display(format!(""));
    }

    fn print_keypair_formatted(&self, key: &String, value: String, formatting: String) {
        match formatting.as_str() {
            "short" => self.logger.display(format!("{}", value)),
            "csv" => {
                let headers = "key,value";
                let pair = format!("{},{}", key, value);
                self.logger.display(format!("{}\n{}\n", headers, pair));
            }
            "table" => {
                let table = Table::new([KeypairItem {
                    key: key.to_string(),
                    value,
                }])
                .to_string();
                self.logger.display(table);
            }
            "json" => {
                let mut json_object: HashMap<String, String> = HashMap::new();
                json_object.insert(key.to_string(), value);
                let json_string = json::stringify_pretty(json_object, 4);
                self.logger.display(json_string);
            }
            _ => self.logger.display(format!("{}={}", key, value)),
        }
    }

    fn print_store_formatted(&self, db: HashMap<String, String>, formatting: String) {
        match formatting.as_str() {
            "short" => {
                for (_key, value) in db {
                    let dec_value = crypto::decrypt_string(&self.keypair.1, value);
                    self.logger.display(format!("{}", dec_value));
                }
            }
            "csv" => {
                let headers = "key,value";
                let mut lines: Vec<String> = vec![];
                for (key, value) in db {
                    let dec_value = crypto::decrypt_string(&self.keypair.1, value);
                    lines.push(format!("{},{}\n", key, dec_value));
                }
                self.logger
                    .display(format!("{}\n{}\n", headers, lines.join("")));
            }
            "json" => {
                let json_string = json::stringify_pretty(db, 4);
                self.logger.display(json_string);
            }
            "table" => {
                let mut table_data: Vec<KeypairItem> = vec![];
                for (key, value) in db {
                    let dec_value = crypto::decrypt_string(&self.keypair.1, value);
                    table_data.push(KeypairItem {
                        key: key,
                        value: dec_value,
                    });
                }
                let table = Table::new(&table_data).to_string();
                self.logger.display(table);
            }
            _ => {
                for (key, value) in db {
                    let dec_value = crypto::decrypt_string(&self.keypair.1, value);
                    self.logger.display(format!("{}={}", key, dec_value));
                }
            }
        }
    }

    fn get_data_dir() -> std::path::PathBuf {
        std::env::home_dir()
            .unwrap_or_default()
            .join(".gui-kvstore") //joining with app directory
    }
}
