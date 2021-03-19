use crate::error::Error;
use diesel::{r2d2::ConnectionManager, MysqlConnection};
use std::time::Duration;
use url::Url;

embed_migrations!();

/// A connection pool to a MySQL database.
pub type Pool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

/// MySQL backend implementation
pub struct MysqlBackend {
  connection_pool: Pool,
}

impl MysqlBackend {
  /// Initialize the connection with the MySQL database.
  pub fn new(url: Url) -> Result<Self, Error> {
    let manager = ConnectionManager::<MysqlConnection>::new(url.as_str());
    let connection_pool: Pool = r2d2::Pool::builder()
      .connection_timeout(Duration::from_secs(5))
      .build(manager)?;

    Ok(Self { connection_pool })
  }

  /// Retrieve a connection from the connection pool.
  pub fn get_connection(
    &self,
  ) -> Result<r2d2::PooledConnection<ConnectionManager<MysqlConnection>>, Error> {
    Ok(self.connection_pool.get()?)
  }
}
