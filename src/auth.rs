// let spotify_enum = init_spotify(&cookies).auto_reauth();
//     let spotify = match spotify_enum {
//         Ok(spotify) => spotify,
//         Err(err) => {
//             remove_cache_path(cookies);
//             return AppResponse::Redirect(Redirect::to("/"));
//         }
//     };


// let spotify_enum: ClientResult<> = init_spotify(&cookies).auto_reauth();
//         let spotify = match spotify_enum {
//             Ok(spotify) => spotify,
//             Err(err) => {
//                 remove_cache_path(cookies);
//                 return AppResponse::Redirect(Redirect::to("/"));
//             }
//         };

    // if !token.is_expired() {
    //     let spotify = AuthCodeSpotify::from_token(token);
    // }
    // else {
    //     let spotify = init_spotify(&cookies).auto_reauth();
    // }