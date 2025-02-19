use std::borrow::Cow;
use std::fmt;

/// A flag for a posting or transaction.
///
/// # Example
/// ```rust
/// use beancount_core::Flag;
/// assert_eq!(Flag::default(), Flag::Okay);
/// assert_eq!(Flag::from("*"), Flag::Okay);
/// assert_eq!(Flag::from("!"), Flag::Warning);
/// assert_eq!(Flag::from(":)"), Flag::Other(":)".into()));
/// ```
// TODO: Make sure that the variant Other("*") can't be created, since Other("*") != Okay
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Flag {
    Okay,
    Warning,
    Other(String),
}

impl Default for Flag {
    fn default() -> Self {
        Flag::Okay
    }
}

impl From<&str> for Flag {
    fn from(s: &str) -> Self {
        match &*s {
            "*" | "txn" => Flag::Okay,
            "!" => Flag::Warning,
            _ => Flag::Other(s.to_owned()),
        }
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Flag::Okay => write!(f, "*"),
            Flag::Warning => write!(f, "!"),
            Flag::Other(s) => write!(f, "{}", s),
        }
    }
}
