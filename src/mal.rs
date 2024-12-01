use mal_api::{anime::error::AnimeApiError, oauth::{Authenticated, OauthClient, RedirectResponse}, prelude::{AnimeApi, AnimeApiClient, AnimeList, GetAnimeList}};
use sqlx::{Pool, Sqlite};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};
use std::time::Duration;
use indicatif::ProgressBar;

use crate::db;

const MAL_CLIENT_ID: &'static str = env!("MAL_CLIENT_ID");
const MAL_REDIRECT_PORT: &'static str = env!("MAL_REDIRECT_PORT");
const MAL_REDIRECT_URL: &'static str = env!("MAL_REDIRECT_URL");
const RESPONSE_TEMPLATE: &str = include_str!("response_template.html");
const BUFFER_SIZE: usize = 1024;

pub async fn auth(db: Pool<Sqlite>) -> OauthClient<Authenticated> {
    let bar = ProgressBar::new_spinner().with_message("🔑 Authenticating");
    bar.enable_steady_tick(Duration::from_millis(100));

    if let Some(auth) = try_conf_login(db.clone()).await {
        bar.finish();
        return auth
    }

    // Now trying browser login
    let mut oauth_client = match OauthClient::new(MAL_CLIENT_ID, None, &MAL_REDIRECT_URL) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to create OauthClient: {}", e);
            std::process::exit(1);
        }
    };

    let auth_url = oauth_client.generate_auth_url();

    open::that(auth_url).expect("Failed to open browser");

    let code = match catch_callback().await {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Failed to get code:\n{}", err);
            std::process::exit(1);
        },
    };

    let response = match RedirectResponse::try_from(code) {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Failed to parse RedirectResponse: {}", err);
            std::process::exit(1);
        }
    };

    // Authentication process
    let result = oauth_client.authenticate(response).await;
    let authenticated_oauth_client = match result {
        Ok(client) => {
            let client = match client.refresh().await {
                Ok(refreshed) => refreshed,
                Err(err) => {
                    eprintln!("Failed to refresh token: {}", err);
                    std::process::exit(1);
                }
            };
            client
        }
        Err(e) => panic!("Failed: {}", e),
    };

    // Save credentials to config to be re-used later
    save_config(db, &authenticated_oauth_client).await;

    bar.finish();

    authenticated_oauth_client
}

async fn save_config(db: Pool<Sqlite>, client: &OauthClient<Authenticated>) {
    let mut conn = match db.acquire().await {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Failed to acquire database connection: {}", err);
            std::process::exit(1);
        }
    };

    // clear table
    let _ = sqlx::query!("DELETE FROM mal_settings").execute(&mut *conn).await;

    // insert new data
    let time = match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(err) => {
            eprintln!("Failed to calculate system time: {}", err);
            std::process::exit(1);
        }
    };
    let expires_at: i64 = (client.get_expires_at() + time) as i64;
    let access_token = client.get_access_token_secret();
    let refresh_token = client.get_refresh_token_secret();

    let _ = sqlx::query!("INSERT INTO mal_settings (mal_access_token, mal_refresh_token, mal_token_expires_at) VALUES (?, ?, ?)",
        access_token, refresh_token, expires_at)
        .execute(&mut *conn).await;
}

async fn try_conf_login(db: Pool<Sqlite>) -> Option<OauthClient<Authenticated>> {
    let mut conn = match db.acquire().await {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Failed to acquire database connection: {}", err);
            return None;
        }
    };

    let settings: Option<db::MalSettings> = match sqlx::query_as!(
        db::MalSettings,
        "SELECT * FROM mal_settings"
    ).fetch_optional(&mut *conn).await {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("Failed to fetch settings: {}", err);
            return None;
        }
    };

    if let Some(settings) = settings {

        let time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let expires_at: i64 = settings.mal_token_expires_at - time as i64;
        if expires_at <= 0 {
            println!("Token exist but it is expired by {} seconds", expires_at.abs());
        }

        let authenticated_client = OauthClient::load_from_values(
            settings.mal_access_token, settings.mal_refresh_token, 
            MAL_CLIENT_ID.to_string(), None, MAL_REDIRECT_URL.to_string(), 
            expires_at as u64,
        );

        match authenticated_client {
            Ok(client) => {
                let refreshed_client = client.refresh().await.unwrap(); // Refresh token
                return Some(refreshed_client)
            },
            Err(err) => {
                println!("No existing Oauth client exists\n{}", err);
                return None;
            }
        }
    } else {
        return None;
    }
}
    
async fn catch_callback() -> Result<String, String> {
    let bar = ProgressBar::new_spinner().with_message("⏳ Waiting for code");
    bar.enable_steady_tick(Duration::from_millis(100));

    let listener = TcpListener::bind(format!("127.0.0.1:{}", MAL_REDIRECT_PORT)).await.map_err(|err| err.to_string() )?;
    let (mut socket, _) = listener.accept().await.map_err(|err| err.to_string() )?;

    let mut buffer = [0; BUFFER_SIZE];
    let n = socket.read(&mut buffer).await.map_err(|err| err.to_string())?;

    let request = String::from_utf8_lossy(&buffer[..n]);

    // Extract the path and query string from the first line of the request
    let path_and_query = request
        .lines()
        .next()  // Get the first line, which should contain the path and query
        .and_then(|line| line.split_whitespace().nth(1)); // Extract the  URL part
    
    if path_and_query.is_none() {
        let err = "URL is empty.";

        // Something went wrong when retriving url

        let response_body = RESPONSE_TEMPLATE
            .replace("{{title}}", "Failed")
            .replace("{{header}}", "Something Went Wrong")
            .replace("{{message}}", err);
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", response_body);

        let response_buff = response.as_bytes();
        socket.write_all(response_buff).await.map_err(|err| err.to_string())?;

        socket.shutdown().await.map_err(|err| err.to_string())?;

        return Err("URL is empty.".into());
    }

    let full_url = format!("{}/{}", MAL_REDIRECT_URL, path_and_query.unwrap().trim_start_matches('/'));

    let response_body = RESPONSE_TEMPLATE
        .replace("{{title}}", "Success")
        .replace("{{header}}", "Operation Successful")
        .replace("{{message}}", "You can now close this tab and return to the app.");

    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", response_body);

    // Respond with a simple HTTP response
    // let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nYou can now close this tab and return to the app.";

    let response_buff = response.as_bytes();
    socket.write_all(response_buff).await.map_err(|err| err.to_string())?;

    // Close the socket connection
    socket.shutdown().await.map_err(|err| err.to_string())?;

    bar.finish();

    Ok(full_url)
}

pub async fn check_name(client: &OauthClient<Authenticated>, name: &str) -> Result<AnimeList, AnimeApiError> {
    let anime_api_client = AnimeApiClient::from(client);
    let query = &GetAnimeList::builder(name).enable_nsfw().limit(5).build().unwrap();
    anime_api_client.get_anime_list(&query).await
}