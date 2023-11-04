use std::sync::Mutex;

pub static DEBUG: Mutex<bool> = Mutex::new(false);

pub fn init(debug: bool) {
    let mut guard = DEBUG.lock().unwrap();
    *guard = debug;
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        paris::output::format_stdout(format!("<blue>[INFO]</> {}", format!($($arg)*)), "\n")
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        paris::output::format_stdout(format!("<red>[ERROR]</> {}", format!($($arg)*)), "\n")
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        paris::output::format_stdout(format!("<yellow>[WARN]</> {}", format!($($arg)*)), "\n")
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if *$crate::logger::DEBUG.lock().unwrap() {
            paris::output::format_stdout(format!("<magenta>[DEBUG]</> {}", format!($($arg)*)), "\n")
        }
    }
}

#[macro_export]
macro_rules! measure_time {
    ($label:expr, $code:expr) => {
        let start_time = std::time::Instant::now();
        $code;
        $crate::debug!("{} took {}ms", $label, start_time.elapsed().as_millis());
    };
}
