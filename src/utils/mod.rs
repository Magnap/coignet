pub mod cached;

use failure::{Error, Fail};
pub fn error_type<T: Fail>(e: Error) -> Result<(), Error> {
    e.downcast::<T>()?;
    Ok(())
}
