use super::establish_connection;
use diesel::result::Error;

pub async fn check_connection() -> Result<(), Error> {
    let _conn = establish_connection().await;
    return Ok(());
}
