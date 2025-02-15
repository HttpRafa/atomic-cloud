#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::generated::plugin::system::log::log_string($crate::generated::plugin::system::log::Level::Debug, format!($($arg)*).as_str()).await
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::generated::plugin::system::log::log_string($crate::generated::plugin::system::log::Level::Info, format!($($arg)*).as_str()).await
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::generated::plugin::system::log::log_string($crate::generated::plugin::system::log::Level::Warn, format!($($arg)*).as_str()).await
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::generated::plugin::system::log::log_string($crate::generated::plugin::system::log::Level::Error, format!($($arg)*).as_str()).await
    };
}
