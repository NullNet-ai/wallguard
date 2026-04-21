use sqlx::{postgres::PgPoolOptions, PgPool};

/// Create a PostgreSQL connection pool and apply pending migrations.
///
/// The caller is expected to exit on error; this function returns an `Err`
/// so that `main` can log the cause before exiting, rather than panicking.
pub async fn create_pool(database_url: &str) -> Result<PgPool, Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(Error::Connect)?;

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .map_err(Error::Migrate)?;

    Ok(pool)
}

/// Fatal startup errors from the database layer.
#[derive(Debug)]
pub enum Error {
    Connect(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect(e)  => write!(f, "database connection failed: {e}"),
            Self::Migrate(e)  => write!(f, "database migration failed: {e}"),
        }
    }
}

impl std::error::Error for Error {}
