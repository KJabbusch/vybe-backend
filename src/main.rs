use rspotify::{
    model::{Country, Market, SearchType, TrackId, Id},
    prelude::*,
    ClientCredsSpotify, Credentials,
};



fn main() {
    let creds = Credentials::from_env().unwrap();

    let mut spotify = ClientCredsSpotify::new(creds);

    // Obtaining the access token. Requires to be mutable because the internal
    // token will be modified. We don't need OAuth for this specific endpoint,
    // so `...` is used instead of `prompt_for_user_token`.
    spotify.request_token().unwrap();

    // Running the requests
    // let birdy_uri = AlbumId::from_uri("spotify:album:0sNOF9WDwhWunNAHPD3Baj").unwrap();
    // let albums = spotify.album(&birdy_uri);
    // println!("Response: {:#?}", albums);
    let track_query = "The Killers";
    let result = spotify.search(
        track_query,
        &SearchType::Track,
        Some(&Market::Country(Country::UnitedStates)),
None,
Some(1),
None,
    );
    match result {
        Ok(tracks) => {
            println!("Response: {:#?}", tracks);
        }
        Err(e) => println!("Error: {}", e),
    }

    let track_id = TrackId::from_uri("spotify:track:003vvx7Niy0yvhvHt4a68B").unwrap();
    let track_data = spotify.track_features(&track_id);
    println!("Response: {:#?}", track_data);

    

    let danceability = track_data.as_ref().unwrap().danceability;
    
    println!("Danceability: {}", danceability);
    let tempo = track_data.as_ref().unwrap().tempo;
    let energy = track_data.as_ref().unwrap().energy;
    

}