//! Automatically re-authentication means you only need to authenticate the
//! usual way at most once to get token. Then everytime you send a request to
//! Spotify server, it will check whether the token is expired and automatically
//! re-authenticate by refresh_token if set `Token.token_refreshing` to true.

use chrono::offset::Utc;
use chrono::Duration;
use rspotify::{
    prelude::*, scopes, AuthCodeSpotify,
    Config, Credentials, OAuth,
    model::{Country, Market, SearchType, TrackId, Id, RecommendationsAttribute, ArtistId, UserId, PlaylistId, SearchResult},
};

use tokio;

// Sample request that will follow some artists, print the user's
// followed artists, and then unfollow the artists.
async fn auth_code_do_things(spotify: &AuthCodeSpotify) {
    let user_id = spotify.current_user().unwrap().id;
    println!("current user id: {}", &user_id);
    
    let playlist = spotify
        .user_playlist_create(&user_id, "pwlease werk", Some(true), Some(false), Some("testing!!!!!"))
        // .await
        .expect("bitch aint work");
    println!("{:#?}", playlist);
    
}

async fn the_killers(spotify: AuthCodeSpotify) {
    let track_query = "The Killers";
    let track_query_result = spotify.search(
        track_query,
        &SearchType::Track,
        Some(&Market::Country(Country::UnitedStates)),
        None,
        Some(1),
        None,
    );

    let track_result = match track_query_result {
        Ok(tracks) => {
            match tracks {
                SearchResult::Tracks(tracks) => tracks,
                _ => {return;}
            }
        }
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    let track_id = track_result.items[0].id.as_ref().unwrap();
    let track_data = spotify.track_features(&track_id);

    let danceability = track_data.as_ref().unwrap().danceability;
    let danceability = (danceability * 100.0).round() / 100.0;
    
    let tempo = track_data.as_ref().unwrap().tempo;
    let tempo = tempo.round();

    let energy = track_data.as_ref().unwrap().energy;
    let energy = (energy * 100.0).round() / 100.0;

    let rec_tempo = RecommendationsAttribute::TargetTempo(tempo);
    let rec_energy = RecommendationsAttribute::TargetEnergy(energy);
    let rec_danceability = RecommendationsAttribute::TargetDanceability(danceability);
    
    let artist_id = ArtistId::from_uri("spotify:artist:0C0XlULifJtAgn6ZNCW2eu").unwrap();
    
    let seed_artists = Some([&artist_id]);
    let seed_tracks = Some([track_id]);
    let seed_genres = Some(["pop", "rock", "indie"]);
    let rec_vec = [rec_tempo, rec_energy, rec_danceability];
    let recommendations = spotify.recommendations(rec_vec, seed_artists, seed_genres, seed_tracks, Some(&Market::Country(Country::UnitedStates)), Some(10));

    
}

async fn expire_token<S: BaseClient>(spotify: &S) {
    let token_mutex = spotify.get_token();
    let mut token = token_mutex.lock().unwrap(); // there was an await before unwrap
    let mut token = token.as_mut().expect("Token can't be empty as this point");
    // In a regular case, the token would expire with time. Here we just do
    // it manually.
    let now = Utc::now().checked_sub_signed(Duration::seconds(10));
    token.expires_at = now;
    token.expires_in = Duration::seconds(0);
    // We also use a garbage access token to make sure it's actually
    // refreshed.
    token.access_token = "garbage".to_owned();
}

async fn with_auth(creds: Credentials, oauth: OAuth, config: Config) {
    // In the first session of the application we authenticate and obtain the
    // refresh token.
    println!(">>> Session one, obtaining refresh token and running some requests:");
    let mut spotify = AuthCodeSpotify::with_config(creds.clone(), oauth, config.clone());
    let url = spotify.get_authorize_url(false).unwrap();
    spotify
        .prompt_for_token(&url)
        // .await
        .expect("couldn't authenticate successfully");

    // We can now perform requests
    auth_code_do_things(&spotify).await;

    // Manually expiring the token.
    // expire_token(&spotify).await;

    // Without automatically refreshing tokens, this would cause an
    // authentication error when making a request, because the auth token is
    // invalid. However, since it will be refreshed automatically, this will
    // work.
    // println!(">>> Session two, the token should expire, then re-auth automatically");
    // auth_code_do_things(&spotify).await;
}



#[tokio::main]
async fn main() {
    // You can use any logger for debugging.
    // env_logger::init();

    // Enabling automatic token refreshing in the config
    let config = Config {
        token_refreshing: true,
        ..Default::default()
    };

    // The default credentials from the `.env` file will be used by default.
    
    // let redirect = dotenv::var("RSPOTIFY_REDIRECT_URI").unwrap();
    // let oauth = OAuth {
    //     // redirect_uri: redirect.to_owned(),
    //     scopes: scopes!("user-read-currently-playing", "playlist-modify-private"),
    //     ..Default::default()
    // };
    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!("user-read-currently-playing", "playlist-modify-public")).unwrap();

    with_auth(creds.clone(), oauth, config.clone()).await;
    // AuthCodeSpotify::with_config(creds, oauth, config)
}

