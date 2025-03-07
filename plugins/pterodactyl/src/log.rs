#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::node::plugin::log::log_string($crate::node::plugin::log::Level::Debug, format!($($arg)*).as_str())
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::node::plugin::log::log_string($crate::node::plugin::log::Level::Info, format!($($arg)*).as_str())
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::node::plugin::log::log_string($crate::node::plugin::log::Level::Warn, format!($($arg)*).as_str())
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::node::plugin::log::log_string($crate::node::plugin::log::Level::Error, format!($($arg)*).as_str())
    };
}
