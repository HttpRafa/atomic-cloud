#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::node::driver::log::log_string($crate::node::driver::log::Level::Debug, format!($($arg)*).as_str());
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::node::driver::log::log_string($crate::node::driver::log::Level::Info, format!($($arg)*).as_str());
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::node::driver::log::log_string($crate::node::driver::log::Level::Warn, format!($($arg)*).as_str());
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::node::driver::log::log_string($crate::node::driver::log::Level::Error, format!($($arg)*).as_str());
    };
}
