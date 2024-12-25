use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenvy::dotenv;
use crate::configs::settings::GLOBAL_CONFIG;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn establish_connection() -> PgPool {
    dotenv().ok();
    let config = &*GLOBAL_CONFIG;
    let manager = ConnectionManager::<PgConnection>::new(&config.database);
    Pool::builder().build(manager).expect("Failed to create pool.")
}

pub fn run_migrations(conn: &mut PgConnection) {
    // 自动运行所有待应用的迁移
    match conn.run_pending_migrations(MIGRATIONS) {
        Ok(_) => println!("Migrations applied successfully."),
        Err(e) => eprintln!("Error applying migrations: {}", e),
    }
}

