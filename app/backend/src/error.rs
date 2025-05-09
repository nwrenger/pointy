use std::{fmt, sync::PoisonError};

use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    /// Lock is poisoned
    PoisonedLock,
    /// A checksum mismatch appeared
    Checksum,
    /// No assets found for this platform
    NoAssets,
    /// File System Error
    FileSystem(String),
    /// Library Loading Error
    LibLoading(String),
    /// Conversion Error
    Conversion(String),
    /// Json Serialzing/Desializing Error
    Json(String),
    /// Reqwest related Error
    Reqwest(String),
    /// Tauri related Error
    Tauri(String),
    /// Global Shortcut Error
    Shortcut(String),
    /// Autostart Error
    Autostart(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PoisonedLock => write!(f, "internal lock was poisoned"),
            Error::Checksum => write!(f, "checksum verification failed"),
            Error::NoAssets => write!(f, "no assets found for this platform"),
            Error::FileSystem(e) => write!(f, "file system error: {}", e),
            Error::LibLoading(e) => write!(f, "library loading error: {}", e),
            Error::Conversion(e) => write!(f, "conversion error: {}", e),
            Error::Json(e) => write!(f, "JSON serialization/deserialization error: {}", e),
            Error::Reqwest(e) => write!(f, "network request error: {}", e),
            Error::Tauri(e) => write!(f, "Tauri runtime error: {}", e),
            Error::Shortcut(e) => write!(f, "global shortcut error: {}", e),
            Error::Autostart(e) => write!(f, "autostart configuration error: {}", e),
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Error::PoisonedLock
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::FileSystem(err.to_string())
    }
}

impl From<libloading::Error> for Error {
    fn from(err: libloading::Error) -> Self {
        Error::LibLoading(err.to_string())
    }
}

impl From<std::ffi::IntoStringError> for Error {
    fn from(err: std::ffi::IntoStringError) -> Self {
        Error::Conversion(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err.to_string())
    }
}

impl From<tauri::Error> for Error {
    fn from(err: tauri::Error) -> Self {
        Error::Tauri(err.to_string())
    }
}

impl From<tauri_plugin_global_shortcut::Error> for Error {
    fn from(err: tauri_plugin_global_shortcut::Error) -> Self {
        Error::Shortcut(err.to_string())
    }
}

impl From<global_hotkey::hotkey::HotKeyParseError> for Error {
    fn from(err: global_hotkey::hotkey::HotKeyParseError) -> Self {
        Error::Shortcut(err.to_string())
    }
}

impl From<tauri_plugin_autostart::Error> for Error {
    fn from(err: tauri_plugin_autostart::Error) -> Self {
        Error::Autostart(err.to_string())
    }
}
