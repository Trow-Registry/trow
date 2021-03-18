use std::fmt;

pub struct RepositoryError {
    pub message: String,
}

pub fn from(message: String) -> RepositoryError {
    RepositoryError { message }
}

// Different error messages according to AppError.code
impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// A unique format for debugging output
impl fmt::Debug for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RepositoryError {{ message: {} }}", self.message)
    }
}
