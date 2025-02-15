use std::fmt;
use wmi::WMIError;

#[derive(Debug, Clone)]
pub struct DiskError {
    message: String,
}

impl DiskError {
    pub fn new(message: String) -> Self {
        DiskError { message }
    }
}

impl fmt::Display for DiskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<WMIError> for DiskError {
    fn from(value: WMIError) -> Self {
        DiskError {
            message: value.to_string(),
        }
    }
}