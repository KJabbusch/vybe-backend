use rspotify::{
    model::{Country, Market, SearchType, TrackId, Id, RecommendationsAttribute, ArtistId},
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
    // println!("Response: {:#?}", track_data);

    

    let danceability = track_data.as_ref().unwrap().danceability;
    let danceability = (danceability * 100.0).round() / 100.0;
    
    let tempo = track_data.as_ref().unwrap().tempo;
    let tempo = tempo.round();

    let energy = track_data.as_ref().unwrap().energy;
    let energy = (energy * 100.0).round() / 100.0;

    // println!("Danceability: {} \nTempo: {} \nEnergy: {}", danceability, tempo, energy);
    
    
    // let attributes = RecommendationsAttribute::new(TargetDanceability::danceability, TargetEnergy::energy, TargetTempo::tempo);
    // let recs = recommendations(attributes, seed_artists, seed_genres, seed_tracks, market, limit);
    // println!("{:#?}", recs);

    let rec_tempo = RecommendationsAttribute::TargetTempo(tempo);
    let rec_energy = RecommendationsAttribute::TargetEnergy(energy);
    let rec_danceability = RecommendationsAttribute::TargetDanceability(danceability);
    
    let artist_id = ArtistId::from_uri("spotify:artist:0C0XlULifJtAgn6ZNCW2eu").unwrap();
    
    let seed_artists = Some([&artist_id]);
    let seed_tracks = Some([&track_id]);
    let seed_genres = Some(["pop", "rock", "indie"]);
    let rec_vec = [rec_tempo, rec_energy, rec_danceability];
    let recommendations = spotify.recommendations(rec_vec, seed_artists, seed_genres, seed_tracks, Some(&Market::Country(Country::UnitedStates)), Some(10));
    println!("{:#?}", recommendations);
}