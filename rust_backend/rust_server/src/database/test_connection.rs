use super::establish_connection;
use crate::utils::Error;

pub async fn check_connection() -> Result<(), Error> {
    let conn = establish_connection().await;
    match conn {
        Ok(_) => Ok(()),
        Err(e) => {error!("{}",e); Err(Error::DatabaseError)},
    }
}
