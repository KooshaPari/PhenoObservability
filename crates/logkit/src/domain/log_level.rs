//! Log Level

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Level {
    Trace, Debug, Info, Warn, Error, Fatal,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
            Level::Fatal => "FATAL",
        }
    }
}

impl Default for Level {
    fn default() -> Self { Level::Info }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_returns_uppercase_static_str() {
        assert_eq!(Level::Trace.as_str(), "TRACE");
        assert_eq!(Level::Debug.as_str(), "DEBUG");
        assert_eq!(Level::Info.as_str(), "INFO");
        assert_eq!(Level::Warn.as_str(), "WARN");
        assert_eq!(Level::Error.as_str(), "ERROR");
        assert_eq!(Level::Fatal.as_str(), "FATAL");
    }

    #[test]
    fn display_matches_as_str() {
        for level in [Level::Trace, Level::Debug, Level::Info, Level::Warn, Level::Error, Level::Fatal] {
            assert_eq!(format!("{}", level), level.as_str());
        }
    }

    #[test]
    fn default_is_info() {
        assert_eq!(Level::default(), Level::Info);
    }

    #[test]
    fn ordering_is_severity_based() {
        assert!(Level::Trace < Level::Debug);
        assert!(Level::Debug < Level::Info);
        assert!(Level::Info < Level::Warn);
        assert!(Level::Warn < Level::Error);
        assert!(Level::Error < Level::Fatal);
    }
}
