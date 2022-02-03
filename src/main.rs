use rspotify::{
    model::{AdditionalType, Country, Market},
    prelude::*,
    scopes, AuthCodeSpotify, Credentials, OAuth,
};

#[tokio::main]
async fn main() {
    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!("user-read-currently-playing")).unwrap();
    
    let mut spotify = AuthCodeSpotify::new(creds, oauth);

    // // Obtaining the access token
    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).unwrap();
    
    // // Running the requests
    let market = Market::Country(Country::UnitedStates);
    let additional_types = [AdditionalType::Episode];
    let artists = spotify
        .current_playing(Some(&market), Some(&additional_types));

    println!("Response: {:#?}", artists);
}