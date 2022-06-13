use std::{collections::HashMap, path::Path};
#[path = "log.rs"]
mod log;

// where to save our files. ideally we move this to a config file later
//static DB_DIR: &str = "C:\\var\\gui-kvstore";

pub struct Database {
    db_data: HashMap<String, String>,
    db_file_path: String,
    logger: log::Logger,
    pub name: String,
}

impl Database {
    pub fn new(store_name: String, store_path: String, log_debug:bool) -> Result<Database, std::io::Error> {
        let mut logger:log::Logger = log::Logger::new();
        logger.toggle_debug(log_debug);
        //
        let db_file_path = Database::get_store_filename(&store_path, &store_name);
        logger.debug(format!("Store Path: {}", store_path));
        // creating map to save entries into
        let mut db_map = HashMap::new();
        //checking if file exists
        let file_exists = std::path::Path::new(&store_path).is_file();
        //
        if file_exists {
            /*
            let contents = match std::fs::read_to_string(format!("{}.db", store)) {
                Ok(c) => c,
                Err(e) => {
                    return Err(e);
                }
            };
            */
            let contents: String = std::fs::read_to_string(store_path)?; //the question mark here is equivalent to the commented block above
                                                                         // reading each line of the file and saving them to the map
            for line in contents.lines() {
                //.lines here returns slices or views to the lines, which is represented by the &str
                let mut chunks = line.splitn(2, '\t');
                let key: &str = chunks.next().expect("no key!"); // getting key as a string slice
                let value: &str = chunks.next().expect("no value!"); // getting value as a string slice
                db_map.insert(key.to_owned(), value.to_owned()); // using to_owned() copies the strings to an owned value
            }
        } else {
            // create file
            //std::fs::File::create(&db_file_path).expect("create failed");
            let db_path = Path::new(&db_file_path);
            std::fs::create_dir_all(db_path.clone().parent().unwrap())?;
            std::fs::File::create(&db_file_path).expect("create failed");
        }

        // returning the Database struct wrapped in an Ok
        Ok(Database {
            db_data: db_map,
            name: store_name,
            db_file_path,
            logger,
        })
    }

    pub fn insert(&mut self, key_arg: String, value_arg: String) {
        self.db_data.insert(key_arg, value_arg);
    }

    pub fn read(&mut self, key: String) -> String {
        self.db_data.get(&key).unwrap_or(&String::from("")).to_owned()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.logger.debug(format!("flushing db: {}", self.name));
        let mut contents = String::new();
        for (key, value) in &self.db_data {
            contents.push_str(&Database::format_keypair(key, value));
        }
        std::fs::write(Database::get_store_filename(&self.db_file_path, &self.name), contents)
    }

    /**
     * Iterates over the DB_DIR folder and prints all stores found
     */
    pub fn print_stores(&self) {
        let db_path = Path::new(&self.db_file_path).parent();
        self.logger.debug(format!("reading stores in: {}", db_path.unwrap().display()));
        let paths = std::fs::read_dir(db_path.unwrap()).unwrap();
        for path in paths {
            self.logger.display(format!(
                "Store Name: {}",
                path.unwrap().path().file_name().unwrap_or_default().to_str().unwrap_or("Unknown")
            ));
        }
    }

    pub fn print_store(&self) {
        self.logger.display("Printing all stores...".to_string());
        for (key, value) in &self.db_data {
            self.logger.display(format!(
                "{}:{}",
                key,
                value
            ));
        }
    }

    /**
     * returns the file path for a db storage with store_name
     */
    fn get_store_filename(store_path:&str, store_name: &str) -> String {
        format!("{}/{}.db", store_path, store_name)
    }

    /**
     * function to format the key-pair into a string
     */
    fn format_keypair(key: &str, value: &str) -> String {
        format!("{}\t{}\n", key, value)
    }
}

/**
 * Implementing the Drop trait for the Database struct
 * This will be called whenever the struct is about to go out of memory
 * In this case it will flush the database, persisting it in the file
 */
impl Drop for Database {
    fn drop(&mut self) {
        //using underscore binding here to ignore the result
        let _ = self.flush();
    }
}
