use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use crate::dir::get_app_dir;

#[cfg(not(debug_assertions))]
fn get_databse_url() -> String {
    if let Some(path) = get_app_dir().to_str() {
        format!("sqlite://{}/{}", path, env!("PRODUCTION_DATABASE_URL"))
    } else {
        panic!("Could not convert path to str");
    }
}

#[cfg(debug_assertions)]
fn get_databse_url() -> String {
    env!("DATABASE_URL").into()
}

#[allow(unused_variables)]
pub async fn connect() -> Result<Pool<Sqlite>, sqlx::Error> {
    let url= get_databse_url();

    let pool = SqlitePoolOptions::new().connect(url.as_str()).await?; 
    
    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}

#[derive(Debug, sqlx::FromRow)]
pub struct Anime {
    pub id: i64,
    pub mal_id: i64,
    pub name: String,
    pub cover_art: Option<Vec<u8>>,
    pub thumbnail: Option<Vec<u8>>,
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
    pub thumbnail: Option<Vec<u8>>,
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

