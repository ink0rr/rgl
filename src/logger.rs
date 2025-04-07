use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, MutexGuard, OnceLock,
    },
};

static DEBUG_FLAG: AtomicBool = AtomicBool::new(false);

fn get_logger() -> MutexGuard<'static, paris::Logger<'static>> {
    static LOGGER: OnceLock<Mutex<paris::Logger<'static>>> = OnceLock::new();
    let logger = LOGGER.get_or_init(|| Mutex::new(paris::Logger::new()));
    logger.lock().unwrap()
}

pub struct Logger;

impl Logger {
    pub fn log<T: Display>(message: T) {
        get_logger().log(message);
    }

    pub fn info<T: Display>(message: T) {
        Logger::log(format!("<blue>[INFO]</> {}", message));
    }

    pub fn warn<T: Display>(message: T) {
        Logger::log(format!("<yellow>[WARN]</> {}", message));
    }

    pub fn error<T: Display>(message: T) {
        Logger::log(format!("<red>[ERROR]</> {}", message));
    }

    pub fn get_debug() -> bool {
        DEBUG_FLAG.load(Ordering::Relaxed)
    }

    pub fn set_debug(debug: bool) {
        DEBUG_FLAG.store(debug, Ordering::Relaxed);
    }

    pub fn debug<T: Display>(message: T) {
        if DEBUG_FLAG.load(Ordering::Relaxed) {
            Logger::log(format!("<magenta>[DEBUG]</> {}", message))
        }
    }

    pub fn loading<T: Display>(message: T) {
        get_logger().loading(message);
    }

    pub fn success<T: Display>(message: T) {
        Logger::log(format!("<green>[DONE]</> {}", message));
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logger::Logger::log(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logger::Logger::info(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logger::Logger::warn(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logger::Logger::error(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::Logger::debug(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! loading {
    ($($arg:tt)*) => {
        $crate::logger::Logger::loading(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        $crate::logger::Logger::success(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! measure_time {
    ($label:expr, $code:expr) => {
        let start_time = std::time::Instant::now();
        $code;
        $crate::debug!("{}: {}ms", $label, start_time.elapsed().as_millis());
    };
}
