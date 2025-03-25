use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;

pub type PgPool = diesel_async::pooled_connection::bb8::Pool<diesel_async::AsyncPgConnection>;

/// Get a connection pool to the Postgres database
pub async fn get_connection_pool(database_url: String) -> PgPool {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .await
        .expect("Could not build Postgres DB connection pool")
}
