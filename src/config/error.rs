#[derive(Debug)]
pub enum ConfigError {
    InvalidConfig(String),
    InternalError(String),
    OpenFile(::std::io::Error),
    SyntaxError(String),
}

use self::ConfigError::InternalError;
use self::ConfigError::InvalidConfig;
use self::ConfigError::OpenFile;
use self::ConfigError::SyntaxError;

impl PartialEq for ConfigError {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (InvalidConfig(a), InvalidConfig(b)) => a == b,
            (InternalError(a), InternalError(b)) => a == b,
            (&OpenFile(_), &OpenFile(_)) => false,
            (SyntaxError(a), SyntaxError(b)) => a == b,
            (_, _) => false,
        }
    }
}
