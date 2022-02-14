//! Automatically re-authentication means you only need to authenticate the
//! usual way at most once to get token. Then everytime you send a request to
//! Spotify server, it will check whether the token is expired and automatically
//! re-authenticate by refresh_token if set `Token.token_refreshing` to true.

use chrono::offset::Utc;
use chrono::Duration;
use rspotify::{
    prelude::*, scopes, AuthCodeSpotify,
    Config, Credentials, OAuth,
    model::{Country, Market, SearchType, TrackId, Id, RecommendationsAttribute, ArtistId, 
        UserId, PlaylistId, SearchResult, SimplifiedTrack, idtypes::PlayableId, Page, FullTrack, FullPlaylist},
};
use std::io;
use std::collections::HashMap;
use std::iter::Iterator;
use tokio;

// Sample request that will follow some artists, print the user's
// followed artists, and then unfollow the artists.
async fn auth_code_do_things(spotify: AuthCodeSpotify) {
    let user_id = spotify.current_user().unwrap().id;
    println!("current user id: {}", &user_id);

    let playlist_name = get_playlist_from_user();
    
    let playlist: FullPlaylist = spotify
        .user_playlist_create(&user_id, &playlist_name, Some(true), Some(false), Some("testing!!!!!"))
        // .await
        .expect("bitch aint work");
    // println!("{:#?}", playlist);
    let playlist_id = playlist.id;
    let tracks = the_killers(&spotify);
    
    let playable: Vec<&dyn PlayableId> = tracks
        .iter()
        .map(|id| id as &dyn PlayableId)
        .collect::<Vec<&dyn PlayableId>>();
    
    let final_try =spotify.playlist_add_items(&playlist_id, playable, Some(0));
    println!("{:#?}", final_try);
    // playable
}

fn get_artist_from_user() -> String {
    let mut artist_name = String::new();
    println!("Enter artist name: ");
    io::stdin().read_line(&mut artist_name).expect("Failed to read line");
    artist_name.trim().to_string()
}

fn get_song_from_user() -> String {
    let mut song_name = String::new();
    println!("Enter song name: ");
    io::stdin().read_line(&mut song_name).expect("Failed to read line");
    song_name.trim().to_string()
}

fn get_playlist_from_user() -> String {
    let mut playlist_name = String::new();
    println!("Enter name for your vybe!: ");
    io::stdin().read_line(&mut playlist_name).expect("Failed to read line");
    playlist_name.trim().to_string()
}

fn hash_map_from_tracks(tracks: Page<FullTrack>) -> HashMap<String, String> {
    let mut track_map = HashMap::new();
    for (i, track) in tracks.items.into_iter().enumerate() {
        track_map.insert(format!("{}", i), track.name);
    };
    track_map

}

fn the_killers(spotify: &AuthCodeSpotify) -> Vec<TrackId> {
    // we need to ask user for artist
    let track_query = get_artist_from_user();
    let track_query_result = spotify.search(
        &track_query,
        &SearchType::Track,
        Some(&Market::Country(Country::UnitedStates)),
        None,
        Some(10),
        None,
    );

    let track_result: Page<FullTrack> = match track_query_result {
        Ok(tracks) => {
            match tracks {
                SearchResult::Tracks(tracks) => tracks,
                _ => panic!("Unexpected result"),
            }
        }
        Err(err) => {
            println!("Error: {}", err);
            return Vec::new();
        }
    };
    let track_result_copy = track_result.clone();

    let track_map = hash_map_from_tracks(track_result_copy);
    println!("{:#?}", track_map);
    // for track in &track_result_copy.items {
    //     println!("{}", track.name);
    // }
    let another_track_copy = track_result.items.clone();
    let track_name = get_song_from_user();

    let track_id = &track_result.items.into_iter().find(|track| track.name == track_name).unwrap().id.unwrap();
    let artist_id = another_track_copy.into_iter().find(|track| track.name == track_name).unwrap().artists.into_iter().find(|artist| artist.name == track_query).unwrap().id.unwrap();
    
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
    
    let seed_artists = Some([&artist_id]);
    let seed_tracks = Some([track_id]);
    let seed_genres = Some(["pop", "rock", "indie"]);
    let rec_vec = [rec_tempo, rec_energy, rec_danceability];
    let recommendations = spotify.recommendations(rec_vec, seed_artists, seed_genres, seed_tracks, Some(&Market::Country(Country::UnitedStates)), Some(10));

    let songs: Vec<SimplifiedTrack> = match recommendations {
        Ok(recommendations) => recommendations.tracks,
        Err(err) => {
            println!("Error: {}", err);
            return Vec::new();
        }
    };

    let song_list = songs.iter().map(|song| song.id.as_ref().unwrap().clone()).collect::<Vec<TrackId>>();

    song_list
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
    let mut spotify: AuthCodeSpotify = AuthCodeSpotify::with_config(creds.clone(), oauth, config.clone());
    let url = spotify.get_authorize_url(false).unwrap();
    spotify
        .prompt_for_token(&url)
        // .await
        .expect("couldn't authenticate successfully");

    // We can now perform requests
    auth_code_do_things(spotify).await;
}

#[tokio::main]
async fn main() {
    let config = Config {
        token_refreshing: true,
        ..Default::default()
    };

    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!("user-read-currently-playing", "playlist-modify-public")).unwrap();

    with_auth(creds.clone(), oauth, config.clone()).await;
}

