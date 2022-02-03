

use rspotify::{model::AlbumId, prelude::*, ClientCredsSpotify, Credentials};

fn main() {
    let creds = Credentials::from_env().unwrap();

    let mut spotify = ClientCredsSpotify::new(creds);

    // Obtaining the access token. Requires to be mutable because the internal
    // token will be modified. We don't need OAuth for this specific endpoint,
    // so `...` is used instead of `prompt_for_user_token`.
    spotify.request_token().unwrap();

    // Running the requests
    let birdy_uri = AlbumId::from_uri("spotify:album:0sNOF9WDwhWunNAHPD3Baj").unwrap();
    let albums = spotify.album(&birdy_uri);

    println!("Response: {:#?}", albums);
}