// src/util/log.rs

//! Logger Utility - Provides file-based logging for TUI applications
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::{LazyLock,OnceLock};
use chrono::Local;

pub static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

/// Global logger instance


pub static LOGGER: LazyLock<Logger> = LazyLock::new(|| {
    Logger::new("./logs").expect("Failed to initialize logger")
});

/// Log severity levels
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    fn as_str(&self) -> &str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }

    fn filename(&self) -> &str {
        match self {
            LogLevel::Error => "error.log",
            LogLevel::Warn => "warn.log",
            LogLevel::Info => "info.log",
            LogLevel::Debug => "debug.log",
        }
    }
}

/// Logger that writes to separate files by severity
pub struct Logger {
    log_dir: PathBuf,
    error_file: Mutex<File>,
    warn_file: Mutex<File>,
    info_file: Mutex<File>,
    debug_file: Mutex<File>,
}

impl Logger {
    /// Create a new logger with the specified directory
    pub fn new(log_dir: &str) -> std::io::Result<Self> {
        DEBUG_ENABLED.get_or_init(|| {
            std::env::var("DEBUG").unwrap_or_default() == "true"
        });

        let log_dir = PathBuf::from(log_dir);

        // Create logs directory if it doesn't exist
        create_dir_all(&log_dir)?;

        // Create/truncate log files (start fresh each time)
        let error_file = File::create(log_dir.join("error.log"))?;
        let warn_file = File::create(log_dir.join("warn.log"))?;
        let info_file = File::create(log_dir.join("info.log"))?;
        let debug_file = File::create(log_dir.join("debug.log"))?;

        Ok(Self {
            log_dir,
            error_file: Mutex::new(error_file),
            warn_file: Mutex::new(warn_file),
            info_file: Mutex::new(info_file),
            debug_file: Mutex::new(debug_file),
        })
    }

    /// Write a log entry to the appropriate file
    fn write_log(&self, level: LogLevel, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let formatted = format!("[{}] [{}] {}\n", timestamp, level.as_str(), message);

        let file = match level {
            LogLevel::Error => &self.error_file,
            LogLevel::Warn => &self.warn_file,
            LogLevel::Info => &self.info_file,
            LogLevel::Debug => &self.debug_file,
        };

        if let Ok(mut file) = file.lock() {
            let _ = file.write_all(formatted.as_bytes());
            let _ = file.flush();
        }
    }

    /// Log an error message
    pub fn error(&self, message: &str) {
        self.write_log(LogLevel::Error, message);
    }

    /// Log a warning message
    pub fn warn(&self, message: &str) {
        self.write_log(LogLevel::Warn, message);
    }

    /// Log an info message
    pub fn info(&self, message: &str) {
        self.write_log(LogLevel::Info, message);
    }

    /// Log a debug message
    pub fn debug(&self, message: &str) {
        self.write_log(LogLevel::Debug, message);
    }
}

/// Convenience macro for error logging with formatting
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        $crate::util::log::LOGGER.error(&message);
    }};
}

/// Convenience macro for warning logging with formatting
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        $crate::util::log::LOGGER.warn(&message);
    }};
}

/// Convenience macro for info logging with formatting
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        $crate::util::log::LOGGER.info(&message);
    }};
}

/// Convenience macro for debug logging with formatting
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        if *$crate::util::log::DEBUG_ENABLED.get().unwrap_or(&false) {
            let message = format!($($arg)*);
            $crate::util::log::LOGGER.debug(&message);
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_logger_creation() {
        let temp_dir = "./test_logs";
        let logger = Logger::new(temp_dir).expect("Failed to create logger");

        logger.error("Test error");
        logger.warn("Test warning");
        logger.info("Test info");
        logger.debug("Test debug");

        // Check that files were created
        assert!(PathBuf::from(temp_dir).join("error.log").exists());
        assert!(PathBuf::from(temp_dir).join("warn.log").exists());
        assert!(PathBuf::from(temp_dir).join("info.log").exists());
        assert!(PathBuf::from(temp_dir).join("debug.log").exists());

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}
