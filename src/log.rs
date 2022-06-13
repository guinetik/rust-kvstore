pub enum LogLevel {
    VERBOSE, //always want to print
    DEBUG,   //only print if debug
}

pub struct Logger {
    pub is_debug: bool,
}

impl Logger {
    pub fn new() -> Logger {
        Logger { is_debug: false }
    }

    fn log(&self, log_level: LogLevel, message: String) {
        match log_level {
            LogLevel::VERBOSE => {
                println!("{}", message);
            }
            LogLevel::DEBUG => {
                if self.is_debug {
                    println!("DEBUG:\t{}", message);
                }
            }
        }
    }

    pub fn debug(&self, message: String) {
        self.log(LogLevel::DEBUG, message);
    }

    pub fn display(&self, message: String) {
        self.log(LogLevel::VERBOSE, message);
    }

    pub fn toggle_debug(&mut self, e:bool) {
        self.is_debug = e;
    }
}
