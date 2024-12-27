use crate::configs::settings::Config;
use tokio_postgres::NoTls;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: Pool<PostgresConnectionManager<NoTls>>, // 使用连接池
}