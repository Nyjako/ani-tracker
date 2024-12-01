use mal_api::{oauth::{Authenticated, OauthClient}, prelude::{AnimeApiClient, GetSuggestedAnime, GetUserAnimeList, GetUserInformation, GetUserMangaList, MangaApi, MangaApiClient, UserApiClient}};
use sqlx::{Pool, Sqlite};

mod mal;
mod db;
mod video;
mod dir;

#[tokio::main]
async fn main() {
    let db: Pool<Sqlite>                   = db::connect().await.unwrap();
    let client: OauthClient<Authenticated> = mal::auth( db.clone() ).await;
    
    let output = dir::full_scan(db.clone()).await;

    endpoints(&client).await;
}

async fn endpoints(oauth_client: &OauthClient<Authenticated>) {
    let anime_api_client = AnimeApiClient::from(oauth_client);
    let manga_api_client = MangaApiClient::from(oauth_client);
    let user_api_client = UserApiClient::from(oauth_client);

    let query = GetSuggestedAnime::builder()
        .fields(&mal_api::anime::all_common_fields())
        .limit(5)
        .build();
    let response = anime_api_client.get_suggested_anime(&query).await;
    if let Ok(response) = response {
        println!("Response: {}\n", response); 
    }

    let query = &GetUserAnimeList::builder("@me").enable_nsfw().limit(5).build().unwrap();
    let response = anime_api_client.get_user_anime_list(query).await;
    if let Ok(response) = response {
        println!("Response: {}\n", response);
    }

    let query = &GetUserMangaList::builder("@me").enable_nsfw().limit(5).build().unwrap();
    let response = manga_api_client.get_user_manga_list(query).await;
    if let Ok(response) = response {
        println!("Response: {}\n", response);
    }

    let user_fields = mal_api::user::all_fields();
    let query = GetUserInformation::new(Some(&user_fields));
    let response = user_api_client.get_my_user_information(&query).await;
    if let Ok(response) = response {
        println!("Response: {}\n", response);
    }
}