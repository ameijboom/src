use git2::{Config, Error, ErrorClass, ErrorCode, Signature};

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

pub fn signature(config: &Config) -> Result<Signature<'_>, git2::Error> {
    let name = config.get_string("user.name").optional()?;
    let email = config.get_string("user.email")?;
    Signature::now(&name.unwrap_or_default(), &email)
}
