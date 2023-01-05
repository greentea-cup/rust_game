use fern::{log_file, Dispatch, InitError};
use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! __logger_format_args_ {
    ($($arg:tt)*) => {
        format_args!($($arg)*)
    };
}

#[macro_export]
macro_rules! __logger_format_args {
    ($($arg:tt)*) => {
        format_args!("{} {}", std::panic::Location::caller(), __logger_format_args_!($($arg)+))
    };
}

#[macro_export(local_inner_macros)]
macro_rules! log {
    (target: $target:expr, $lvl:expr, $($key:tt = $value:expr),+; $($arg:tt)+) => (
        log::log!(target: target, $lvl, $($key = $value),+; "{}", __logger_format_args!($($arg)+))
    );

    (target: $target:expr, $lvl:expr, $($arg:tt)+) => (
        log::log!(target: $target, $lvl, "{}", __logger_format_args!($($arg)+))
    );

    ($lvl:expr, $($arg:tt)+) => (
        log::log!($lvl, "{}", __logger_format_args!($($arg)+))
    );
}

#[macro_export(local_inner_macros)]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => (
        log::error!(target: $target, "{}", __logger_format_args!($($arg)+))
    );

    ($($arg:tt)+) => (
        log::error!("{}", __logger_format_args!($($arg)+))
    )
}

#[macro_export(local_inner_macros)]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)+) => (
        log::warn!(target: $target, "{}", __logger_format_args!($($arg)+))
    );

    ($($arg:tt)+) => (
        log::warn!("{}", __logger_format_args!($($arg)+))
    )
}

#[macro_export(local_inner_macros)]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => (
        log::info!("{}", __logger_format_args!($($arg)+))
    );

    ($($arg:tt)+) => (
        log::info!("{}", __logger_format_args!($($arg)+))
    )
}

#[macro_export(local_inner_macros)]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => (
        log::debug!(target: $target, "{}", __logger_format_args!($($arg)+))
    );

    ($($arg:tt)+) => (
        log::debug!("{}", __logger_format_args!($($arg)+))
    )
}

#[macro_export(local_inner_macros)]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => (
        log:trace!(target: $target, "{}", __logger_format_args!($($arg)+))
    );

    ($($arg:tt)+) => (
        log::trace!("{}", __logger_format_args!($($arg)+))
    )
}

pub(crate) fn setup_logger(log_dir: &Path) -> Result<(), InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {} {} {}",
                record.level(),
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(log_file(get_log_path(log_dir))?)
        .apply()?;
    Ok(())
}

fn get_log_path(log_dir: &Path) -> PathBuf {
    log_dir.join(chrono::Local::now().to_string() + ".log")
}
