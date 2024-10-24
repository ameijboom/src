use git2::{Error, ErrorClass, ErrorCode};

pub mod commit;
pub mod index;
pub mod signer;
pub mod status;

pub trait Optional<T> {
    fn optional(self) -> Result<Option<T>, Error>;
}

impl<T> Optional<T> for Result<T, git2::Error> {
    fn optional(self) -> Result<Option<T>, Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.code() == ErrorCode::NotFound && e.class() == ErrorClass::Config => {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
}
