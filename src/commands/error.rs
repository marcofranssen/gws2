use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    Git2Error(git2::Error),
    NoBranchNameFound,
    RepositoryMissing,
}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Error {
        Error::Git2Error(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::Git2Error(e) => fmt::Display::fmt(e, formatter),
            Error::NoBranchNameFound => write!(formatter, "No branch name found"),
            Error::RepositoryMissing => write!(formatter, "Repository missing"),
        }
    }
}
