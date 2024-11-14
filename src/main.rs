use mal_api::oauth::{OauthClient, RedirectResponse};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};

const MAL_CLIENT_ID: &'static str = env!("MAL_CLIENT_ID");
const MAL_REDIRECT_PORT: &'static str = env!("MAL_REDIRECT_PORT");

const BUFFER_SIZE: usize = 1024;

async fn catch_callback() -> Result<String, String> {
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
        // Something went wrong when retriving url
        return Err("URL is empty.".into());
    }

    let full_url = format!("http://localhost:{}/{}", MAL_REDIRECT_PORT, path_and_query.unwrap().trim_start_matches('/'));

    // Respond with a simple HTTP response
    // let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nYou can now close this tab and return to the app.";

    // Respond with a simple HTML
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
        <!DOCTYPE html>\
        <html lang=\"en\">\
        <head>\
            <meta charset=\"UTF-8\">\
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\
            <title>Success</title>\
            <style>\
                body { font-family: Arial, sans-serif; text-align: center; padding: 50px; background-color: #f0f0f0; }\
                h1 { color: #333; }\
                p { font-size: 1.2em; color: #555; }\
            </style>\
        </head>\
        <body>\
            <h1>Operation Successful</h1>\
            <p>You can now close this tab and return to the app.</p>\
        </body>\
        </html>";

    let response_buff = response.as_bytes();
    socket.write_all(response_buff).await.map_err(|err| err.to_string())?;

    // Close the socket connection
    socket.shutdown().await.map_err(|err| err.to_string())?;

    Ok(full_url)
}

#[tokio::main]
async fn main() {
    let mut oauth_client = OauthClient::new(MAL_CLIENT_ID.to_string(), None, format!("http://localhost:{}/callback", MAL_REDIRECT_PORT)).unwrap();

    let auth_url = oauth_client.generate_auth_url();
    // println!("Visit this URL: {}\n", auth_url);
    open::that(auth_url).expect("Failed to open browser");

    let code = match catch_callback().await {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Failed to get code:\n{}", err);
            std::process::exit(1);
        },
    };

    let response = RedirectResponse::try_from(code).unwrap();

    // Authentication process
    let result = oauth_client.authenticate(response).await;
    let authenticated_oauth_client = match result {
        Ok(t) => {
            println!("Got token: {:?}\n", t.get_access_token_secret());

            let t = t.refresh().await.unwrap();
            println!("Refreshed token: {:?}", t.get_access_token_secret());
            t
        }
        Err(e) => panic!("Failed: {}", e),
    };

    // Save credentials to config to be re-used later
    let _ = authenticated_oauth_client.save_to_config(".mal/config.toml");
}