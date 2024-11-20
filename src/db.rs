use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

pub async fn connect() -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .connect(env!("DATABASE_URL")).await?; 
    
    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}

#[derive(Debug, sqlx::FromRow)]
pub struct Anime {
    pub id: i64,
    pub mal_id: i64,
    pub name: String,
    pub cover_art: String,
    pub description: Option<String>,
    pub total_episodes: i64,
    pub status: String,
    pub directory_id: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Episodes {
    pub id: i64,
    pub anime_id: i64,
    pub episode_number: i64,
    pub file_path: String,
    pub length: i64,
    pub watched_time: i64,
    pub is_watched: bool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Directories {
    pub id: i64,
    pub path: String,
    pub dir_type: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Settings {
    pub key: String,
    pub value: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct MalSettings {
    pub mal_access_token: String,
    pub mal_refresh_token: String,
    pub mal_token_expires_at: i64,
}

