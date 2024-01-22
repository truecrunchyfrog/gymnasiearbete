use serde::Serialize;

use crate::database::check_connection;

#[derive(Serialize)]
pub struct ServerStatus {
    pub database_connection_status: bool,
    pub docker_connection_status: bool,
}

impl ServerStatus {
    pub async fn new() -> Self {
        let database_connection = check_connection().await;
        let database_connection_status = match database_connection {
            Ok(()) => true,
            Err(_) => false,
        };
        let docker_connection_status = crate::check_docker_socket();
        Self {
            database_connection_status,
            docker_connection_status,
        }
    }
}
