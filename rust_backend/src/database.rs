use dotenv;
use sqlx::postgres::PgConnectOptions;
use sqlx::PgPool;

pub async fn connect_to_db() -> Result<PgPool, sqlx::Error> {
    dotenv::dotenv().ok();
    let url = std::env::var("DATABASE_URL").unwrap();
    let port = std::env::var("DATABASE_PORT").unwrap();
    let username = std::env::var("DATABASE_USER").unwrap();
    let password = std::env::var("DATABASE_PASSWORD").unwrap();
    let database = std::env::var("DATABASE_DBNAME").unwrap();
    let pool_options = PgConnectOptions::new()
        .host(&url)
        .port(port.parse().unwrap())
        .username(&username)
        .database(&database)
        .password(&password);
    Ok(PgPool::connect_with(pool_options).await?)
}

#[derive(sqlx::FromRow)]
struct User {
    name: String,
}

pub async fn print_users(db: &PgPool) {
    let users: Vec<User> = sqlx::query_as("SELECT * FROM users")
        .fetch_all(db)
        .await
        .expect("Failed to find users");
    for u in users {
        println!("User: [ {} ]", u.name)
    }
}
