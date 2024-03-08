use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ArticleFsError {
    FailedToDelete,
}

impl fmt::Display for ArticleFsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::FailedToDelete => write!(f, "Failed to delete article from file system!"),
        }
    }
}

impl error::Error for ArticleFsError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::FailedToDelete => None,
        }
    }
}
