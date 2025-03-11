//! This module provides a custom error structure for disk-related operations
//! in the system.
//!
//! `DiskError` serves as a unified error type that can represent various
//! kinds of errors that might occur during disk operations, including
//! Windows Management Instrumentation (WMI) errors.

use std::fmt;
use wmi::WMIError;

/// Represents a disk-related operation error.
///
/// This structure encapsulates errors that may occur when interacting
/// with storage devices or querying disk information through WMI.
#[derive(Debug, Clone)]
pub struct DiskError {
    /// Descriptive error message.
    message: String,
}

impl DiskError {
    /// Creates a new instance of `DiskError` with the specified message.
    ///
    /// # Arguments
    ///
    /// * `message` - A string describing the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use win_disk_info::DiskError;
    ///
    /// let error = DiskError::new(String::from("Failed to access disk"));
    /// ```
    pub fn new(message: String) -> Self {
        DiskError { message }
    }
}

/// Implementation of the `Display` trait for text representation of the error.
impl fmt::Display for DiskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Automatic conversion from `WMIError` to `DiskError`.
///
/// Allows capturing and converting WMI errors to our custom error type,
/// facilitating consistent error propagation throughout the application.
impl From<WMIError> for DiskError {
    fn from(value: WMIError) -> Self {
        DiskError {
            message: value.to_string(),
        }
    }
}

/// Implementation of the `std::error::Error` trait for `DiskError`.
/// 
/// This allows treating `DiskError` as a standard error type and
/// provides additional error handling capabilities.
impl std::error::Error for DiskError {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}