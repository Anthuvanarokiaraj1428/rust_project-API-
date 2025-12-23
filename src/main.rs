use serde::{Deserialize, Serialize};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, ACCEPT};
use urlencoding::encode;
use base64::engine::general_purpose;
use base64::Engine;
use tokio;

#[derive(Serialize, Deserialize, Debug)]
struct ExternalUrls {
    spotify: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Artist {
    name: String,
    external_urls: ExternalUrls,
}

#[derive(Serialize, Deserialize, Debug)]
struct Album {
    name: String,
    artists: Vec<Artist>,
    external_urls: ExternalUrls,
}

#[derive(Serialize, Deserialize, Debug)]
struct Track {
    name: String,
    href: String,
    popularity: u32,
    album: Album,
    external_urls: ExternalUrls,
}

#[derive(Serialize, Deserialize, Debug)]
struct Items<T> {
    items: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
struct APIResponse {
    tracks: Items<Track>,
}

fn print_tracks(tracks: Vec<&Track>) {
    for track in tracks {
        println!("Track: {}", track.name);
        println!("Album: {}", track.album.name);
        println!(
            "Artists: {}",
            track
                .album
                .artists
                .iter()
                .map(|artist| artist.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!("Spotify URL: {}", track.external_urls.spotify);
        println!("----------------------------");
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    #[allow(dead_code)]
    expires_in: u64,
}

async fn get_spotify_token(client_id: &str, client_secret: &str) -> String {
    let auth = general_purpose::STANDARD.encode(format!("{}:{}", client_id, client_secret));
    let client = reqwest::Client::new();
    let params = [("grant_type", "client_credentials")];

    let res = client
        .post("https://accounts.spotify.com/api/token")
        .header(AUTHORIZATION, format!("Basic {}", auth))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .expect("Failed to request token");

    let token_res: TokenResponse = res.json().await.expect("Failed to parse token response");
    token_res.access_token
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("cargo run -- <search_query>");
        return;
    }

    let search_query = &args[1];

    // Your Spotify app credentials {client_id} and {client_secret}
    let client_id = "bc5fb4477fbf4ff6a7140028f793388e";
    let client_secret = "f136f8e31efa4019880c942f628d6e8a";

    let auth_token = get_spotify_token(client_id, client_secret).await;

    let url = format!(
        "https://api.spotify.com/v1/search?q={}&type=track&market=IN&limit=5",
        encode(search_query)
    );

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", auth_token))
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .await
        .expect("Failed to send request");

    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<APIResponse>().await {
                Ok(parsed) => print_tracks(parsed.tracks.items.iter().collect()),
                Err(e) => println!("Error parsing JSON: {:?}", e),
            }
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            println!("Unauthorized! Check CLIENT_ID and CLIENT_SECRET.");
        }
        other => {
            panic!("Unexpected response: {:?}", other);
        }
    };
}
