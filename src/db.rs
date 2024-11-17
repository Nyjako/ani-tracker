use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

pub struct DB {
    pool: Pool<Sqlite>
}

impl DB {
    pub async fn new() -> Result<DB, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .connect(env!("DB_URL")).await?; 
        
        sqlx::migrate!().run(&pool).await?;

        Ok(
            DB {
                pool,
            }
        )
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct Directory {
    id: u32,
    path: String,
    dir_type: u32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Anime {
    id: u32,
    name: String,
    path: String,
    root_directory_id: u32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Episode {
    id: u32,
    name: String,
    anime_id: u32,
    path: String
}