//! Automatically re-authentication means you only need to authenticate the
//! usual way at most once to get token. Then everytime you send a request to
//! Spotify server, it will check whether the token is expired and automatically
//! re-authenticate by refresh_token if set `Token.token_refreshing` to true.


use rspotify::{
    prelude::*, scopes, AuthCodeSpotify,
    Config, Credentials, OAuth,
    model::{Country, Market, SearchType, TrackId, RecommendationsAttribute, ArtistId, 
    SearchResult, idtypes::PlayableId, Page, FullTrack, FullArtist, FullPlaylist},
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
        .user_playlist_create(&user_id, &playlist_name, Some(true), Some(false), Some("A vybe ✨!!!"))
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

fn get_artist_from_user(spotify: &AuthCodeSpotify) -> (String, ArtistId) {
    let mut artist_name = String::new();
    println!("Enter artist name: ");
    io::stdin().read_line(&mut artist_name).expect("Failed to read line");
    artist_name.trim().to_string();
    let artist_query_result = spotify.search(
        &artist_name,
        &SearchType::Artist,
        Some(&Market::Country(Country::UnitedStates)),
        None,
        None,
        None,
    ).unwrap();

    let artists = match artist_query_result {
        SearchResult::Artists(tracks) => tracks,
        _ => panic!("Unexpected result"),
    };

    let artist_map = hash_map_from_artists(artists);
    println!("{:#?}", artist_map);
    prompt_artist_from_user(artist_map)
}

fn get_song_from_user(mut track_map: HashMap<String, String>) -> String {
    let mut song_num = String::new();
    println!("Enter number for song: ");
    io::stdin().read_line(&mut song_num).expect("Failed to read line");
    let song_num = song_num.trim();
    let return_song = song_num.clone();
    if track_map.contains_key(song_num) {
        track_map.remove(return_song).unwrap()
    } else {
        println!("Invalid song number");
        get_song_from_user(track_map);
        String::new()
    }
}

fn prompt_artist_from_user(mut artist_map: HashMap<String, (String, ArtistId)>) -> (String, ArtistId) {
    let mut artist_num = String::new();
    println!("Enter number for artist: ");
    io::stdin().read_line(&mut artist_num).expect("Failed to read line");
    let artist_num = artist_num.trim();
    let return_artist = artist_num.clone();
    if artist_map.contains_key(artist_num) {
        artist_map.remove(return_artist).unwrap()
    } else {
        println!("Invalid artist number");
        prompt_artist_from_user(artist_map)
        // (String::new(), ArtistId::new())
    }
}

fn get_playlist_from_user() -> String {
    let mut playlist_name = String::new();
    println!("Enter name for your vybe!: ");
    io::stdin().read_line(&mut playlist_name).expect("Failed to read line");
    playlist_name.trim().to_string()
}

fn hash_map_from_tracks(tracks: Vec<FullTrack>) -> HashMap<String, String> {
    let mut track_map = HashMap::new();
    for (i, track) in tracks.into_iter().enumerate() {
        track_map.insert(format!("{}", i), track.name);
    };
    track_map
}

fn hash_map_from_artists(artists: Page<FullArtist>) -> HashMap<String, (String, ArtistId)> {
    let mut artist_map = HashMap::new();
    for (i, artist) in artists.items.into_iter().enumerate() {
        artist_map.insert(format!("{}", i), (artist.name, artist.id));
    };
    artist_map
}

fn get_seed_genres(spotify: &AuthCodeSpotify, artist_id: &ArtistId) -> HashMap<String, String>{
    let artist_genres = spotify.artist(artist_id).unwrap().genres;
    let mut genre_map = HashMap::new();
    for (i, genre) in artist_genres.into_iter().enumerate() {
        genre_map.insert(format!("{}", i), genre);
    }
    genre_map
}

fn get_genre_from_user(genre_map: & mut HashMap<String, String>) -> Vec<String> {
    let mut genre_vec = Vec::new();
    let loop_num = genre_map.len();
    if genre_map.len() <= 3 {
        for i in 0..loop_num {
            let genre = genre_map.remove(&format!("{}", i)).unwrap();
            genre_vec.push(genre);
        }
    } else {
        println!("{:?}", genre_map);
        for i in 0..3 {
            let mut genre_num = String::new();
            let mut genre_map = genre_map.clone();
            println!("Enter number for genre: ");
            io::stdin().read_line(&mut genre_num).expect("Failed to read line");
            let genre_num = genre_num.trim();
            if genre_map.contains_key(genre_num) {
                genre_vec.push(genre_map.remove(genre_num).unwrap());
            } else {
                println!("Invalid genre number");
                get_genre_from_user(&mut genre_map);
            }
        }
    }
    genre_vec
}

fn gets_top_songs(spotify: &AuthCodeSpotify, artist_id: ArtistId) -> Vec<FullTrack> {
    spotify.artist_top_tracks(&artist_id, &Market::Country(Country::UnitedStates)).unwrap()
}

fn the_killers(spotify: &AuthCodeSpotify) -> Vec<TrackId> {
    // we need to ask user for artist
    let (track_query, artist_id_init) = get_artist_from_user(spotify);

    let tracks_from_artist = gets_top_songs(spotify, artist_id_init);
    // let track_query_result = spotify.search(
    //     &track_query,
    //     &SearchType::Track,
    //     Some(&Market::Country(Country::UnitedStates)),
    //     None,
    //     None,
    //     None,
    // );

    // let track_result: Page<FullTrack> = match track_query_result {
    //     Ok(tracks) => {
    //         match tracks {
    //             SearchResult::Tracks(tracks) => tracks,
    //             _ => panic!("Unexpected result"),
    //         }
    //     }
    //     Err(err) => {
    //         println!("Error: {}", err);
    //         return Vec::new();
    //     }
    // };

    // let track_result_copy = track_result.clone();

    let track_copy = tracks_from_artist.clone();
    let another_track_copy = tracks_from_artist.clone();
    let track_map = hash_map_from_tracks(tracks_from_artist);
    println!("{:#?}", track_map);
    // for track in &track_result_copy.items {
    //     println!("{}", track.name);
    // }
    
    let track_name = get_song_from_user(track_map);

    let track_id = track_copy.into_iter().find(|track| track.name == track_name).unwrap().id.unwrap();
    let artist_id = another_track_copy.into_iter().find(|track| track.name == track_name).unwrap().artists.into_iter().find(|artist| artist.name == track_query).unwrap().id.unwrap();
    
    let track_data = spotify.track_features(&track_id);

    // let danceability = track_data.as_ref().unwrap().danceability;
    // let danceability = (danceability * 100.0).round() / 100.0;
    
    let tempo = track_data.as_ref().unwrap().tempo;
    let tempo = tempo.round();

    let energy = track_data.as_ref().unwrap().energy;
    let energy = (energy * 100.0).round() / 100.0;

    let valence = track_data.as_ref().unwrap().valence;
    let valence = (valence * 100.0).round() / 100.0;

    let acousticness = track_data.as_ref().unwrap().acousticness;
    let acousticness = (acousticness * 100.0).round() / 100.0;

    let instrumentalness = track_data.as_ref().unwrap().instrumentalness;
    let instrumentalness = (instrumentalness * 100.0).round() / 100.0;

    // let mode = track_data.as_ref().unwrap().mode;

    let time_signature = track_data.as_ref().unwrap().time_signature;

    let rec_tempo = RecommendationsAttribute::TargetTempo(tempo);
    let rec_energy = RecommendationsAttribute::TargetEnergy(energy);
    // let rec_danceability = RecommendationsAttribute::TargetDanceability(danceability);
    let rec_valence = RecommendationsAttribute::TargetValence(valence);
    // let rec_mode = RecommendationsAttribute::TargetMode(mode); this is an enummmmmm
    let rec_time = RecommendationsAttribute::TargetTimeSignature(time_signature);
    let rec_instrumentalness = RecommendationsAttribute::TargetInstrumentalness(instrumentalness);
    let rec_acousticness = RecommendationsAttribute::TargetAcousticness(acousticness);

    let mut genre_map = get_seed_genres(spotify, &artist_id);

    let genre_vec = get_genre_from_user(&mut genre_map);
    // let (genre_vec_copy, genre_vec) = i_hate_borrowed_shit(genre_vec);
    let mut new_vec = Vec::<&str>::new();
    for genre in &genre_vec {
        new_vec.push(genre.as_str());
    }

    println!("{:?}", new_vec);

    let seed_artists = Some([&artist_id]);
    let seed_tracks = Some([&track_id]);
    let seed_genres = Some(new_vec);
    let rec_vec = [rec_tempo, rec_energy, rec_valence, rec_time, rec_instrumentalness, rec_acousticness];
    let recommendations = spotify.recommendations(rec_vec, seed_artists, seed_genres, seed_tracks, Some(&Market::Country(Country::UnitedStates)), Some(50));

    let songs = match recommendations {
        Ok(recommendations) => recommendations.tracks.into_iter().map(|track| track.id.unwrap()).collect::<Vec<TrackId>>(),
        Err(err) => {
            println!("Error: {}", err);
            return Vec::new();
        }
    };

    let mut tracks_full = spotify.tracks(&songs, Some(&Market::Country(Country::UnitedStates))).unwrap();
    tracks_full.sort_by(|a, b| a.popularity.cmp(&b.popularity));
    
    let mut track_ids = Vec::<TrackId>::new();
    for i in 0..10 {
        let current = tracks_full[i+20].id.clone().unwrap();
        track_ids.push(current);
    }
    track_ids
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

