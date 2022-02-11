#![feature(proc_macro_hygiene, decl_macro, into_future)]

#[macro_use]
extern crate rocket;

use getrandom::getrandom;
use rocket::http::{Cookie, Cookies};
use rocket::response::Redirect;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use rocket_contrib::templates::Template;
use rocket::request::Request;
use rocket::data::{self, Data, FromData};
use rspotify::{scopes, AuthCodeSpotify, OAuth, Credentials, Config, prelude::*, Token, ClientResult};

use std::borrow::Borrow;
use std::fs;
use std::future::IntoFuture;
use std::{
    collections::HashMap,
    env,
    path::PathBuf,
    sync::{Arc, Mutex},
};




#[derive(Debug, Responder)]
pub enum AppResponse {
    Template(Template),
    Redirect(Redirect),
    Json(JsonValue),
}

const CACHE_PATH: &str = ".spotify_cache/";
// we need to figure out where we send cache to (.spotify_cache)
// then we can create a database to store the access token and refresh token
// and create the ability to replace the access token if expired!

/// Generate `length` random chars
fn generate_random_uuid(length: usize) -> String {
    let alphanum: &[u8] =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes();
    let mut buf = vec![0u8; length];
    getrandom(&mut buf).unwrap();
    let range = alphanum.len();

    buf.iter()
        .map(|byte| alphanum[*byte as usize % range] as char)
        .collect()
}

fn get_cache_path(cookies: &Cookies) -> PathBuf {
    let project_dir_path = env::current_dir().unwrap();
    let mut cache_path = project_dir_path;
    cache_path.push(CACHE_PATH);
    cache_path.push(cookies.get("uuid").unwrap().value());

    cache_path
}

fn create_cache_path_if_absent(cookies: &Cookies) -> PathBuf {
    let cache_path = get_cache_path(cookies);
    if !cache_path.exists() {
        let mut path = cache_path.clone();
        path.pop();
        fs::create_dir_all(path).unwrap();
    }
    cache_path
}

fn remove_cache_path(mut cookies: Cookies) {
    let cache_path = get_cache_path(&cookies);
    if cache_path.exists() {
        fs::remove_file(cache_path).unwrap()
    }
    cookies.remove(Cookie::named("uuid"))
}

fn check_cache_path_exists(cookies: &Cookies) -> bool {
    let cache_path = get_cache_path(cookies);
    cache_path.exists()
}

fn init_spotify(cookies: &Cookies) -> AuthCodeSpotify {
    let config = Config {
        token_cached: true,
        cache_path: create_cache_path_if_absent(cookies),
        token_refreshing: true,
        ..Default::default()
    };

    // Please notice that protocol of redirect_uri, make sure it's http
    // (or https). It will fail if you mix them up.
    let redirect = dotenv::var("RSPOTIFY_REDIRECT_URI").unwrap();
    let oauth = OAuth {
        scopes: scopes!("user-read-currently-playing", "playlist-modify-private"),
        redirect_uri: redirect.to_owned(),
        ..Default::default()
    };
    let creds = Credentials::from_env().unwrap();
    AuthCodeSpotify::with_config(creds, oauth, config)
}

#[get("/callback?<code>")]
fn callback(cookies: Cookies, code: String) -> AppResponse {
    let mut spotify = init_spotify(&cookies);

    match spotify.request_token(&code) {
        Ok(_) => {
            println!("Request user token successful");
            AppResponse::Redirect(Redirect::to("/"))
        }
        Err(err) => {
            println!("Failed to get user token {:?}", err);
            let mut context = HashMap::new();
            context.insert("err_msg", "Failed to get token!");
            AppResponse::Template(Template::render("error", context))
        }
    }
}

#[get("/")]
fn index(mut cookies: Cookies) -> AppResponse {
    let mut context = HashMap::new();

    // The user is authenticated if their cookie is set and a cache exists for
    // them.
    let authenticated = cookies.get("uuid").is_some() && check_cache_path_exists(&cookies);
    if !authenticated {
        cookies.add(Cookie::new("uuid", generate_random_uuid(64)));

        let spotify = init_spotify(&cookies);
        let auth_url = spotify.get_authorize_url(true).unwrap();
        context.insert("auth_url", auth_url);
        return AppResponse::Template(Template::render("authorize", context));
    }

    let cache_path = get_cache_path(&cookies);
    let token = Token::from_cache(cache_path).unwrap();
    let check_token = &token.clone();
    if check_token.is_expired() {
        sign_out(cookies);
        // let mut cookies = Cookies::empty();
        // cookies.add(Cookie::new("uuid", generate_random_uuid(64)));
        // let spotify = init_spotify(&cookies);
        // let auth_url = spotify.get_authorize_url(true).unwrap();
        // context.insert("auth_url", auth_url);
    }
    let spotify = AuthCodeSpotify::from_token(token);
    
    match spotify.me() {
        Ok(user_info) => {
            context.insert(
                "display_name",
                user_info
                    .display_name
                    .unwrap_or_else(|| String::from("Dear")),
            );
            AppResponse::Template(Template::render("index", context.clone()))
        }
        Err(err) => {
            context.insert("err_msg", format!("Failed for {}!", err));
            AppResponse::Template(Template::render("error", context))
        }
    }    
}

#[get("/sign_out")]
fn sign_out(cookies: Cookies) -> AppResponse {
    remove_cache_path(cookies);
    AppResponse::Redirect(Redirect::to("/"))
}

#[get("/playlists")]
fn playlist(cookies: Cookies) -> AppResponse {
    let mut spotify = init_spotify(&cookies);
    if !spotify.config.cache_path.exists() {
        return AppResponse::Redirect(Redirect::to("/"));
    }

    let token = spotify.read_token_cache(false).unwrap();
    spotify.token = Arc::new(Mutex::new(token));
    let playlists = spotify.current_user_playlists()
        .take(50)
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    if playlists.is_empty() {
        return AppResponse::Redirect(Redirect::to("/"));
    }

    AppResponse::Json(json!(playlists))
}

#[get("/me")]
fn me(cookies: Cookies) -> AppResponse {
    let mut spotify = init_spotify(&cookies);
    if !spotify.config.cache_path.exists() {
        return AppResponse::Redirect(Redirect::to("/"));
    }

    spotify.token = Arc::new(Mutex::new(spotify.read_token_cache(false).unwrap()));
    match spotify.me() {
        Ok(user_info) => AppResponse::Json(json!(user_info)),
        Err(_) => AppResponse::Redirect(Redirect::to("/")),
    }
}

#[post("/playlists")]
fn make_playlist(cookies: Cookies) -> AppResponse {
    let mut spotify = init_spotify(&cookies);
    if !spotify.config.cache_path.exists() {
        return AppResponse::Redirect(Redirect::to("/"));
    }

    let playlist = "test";

    let token = spotify.read_token_cache(false).unwrap();
    spotify.token = Arc::new(Mutex::new(token));
    match spotify.me() {
        Ok(user_info) => {
            let mut description = String::new();
            description.push_str(&playlist);
            description.push_str(" vybe!");
            let vybe = spotify.user_playlist_create(&user_info.id, &playlist, Some(true), Some(false), Some(&description));
        }
        Err(err) => {
        } 
    }

    let playlists = spotify.current_user_playlists()
        .take(50)
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    if playlists.is_empty() {
        return AppResponse::Redirect(Redirect::to("/"));
    }

    AppResponse::Json(json!(playlists))
}

// #[post("/me/playlist", data="<playlist>", format="application/json")]
// fn create_playlist(cookies: Cookies, playlist: String) -> AppResponse {
//     let mut spotify = init_spotify(&cookies);
//     let mut context = HashMap::new();
//     let token = spotify.read_token_cache(false).unwrap();
//     spotify.token = Arc::new(Mutex::new(token));

    // match spotify.me() {
    //     Ok(user_info) => {
    //         // let id = user_info.id;
    //         let mut description = String::new();
    //         description.push_str(&playlist);
    //         description.push_str(" vybe!");
    //         let vybe = spotify.user_playlist_create(&user_info.id, &playlist, Some(true), Some(false), Some(&description));
    //         match vybe {
    //             Ok(stuff) => {
    //                 AppResponse::Json(json!(stuff))
    //             }
    //             Err(err) => {
    //                 context.insert("err_msg", format!("Failed for {}!", err));
    //                 AppResponse::Template(Template::render("error", context))
    //             }
    //         }
    //     }
    //     Err(err) => {
    //         println!("Failed to get user info {:?}", err);
    //         AppResponse::Redirect(Redirect::to("/"))
    //     } 
    // }
// }

fn main() {
    rocket::ignite()
        .mount("/", routes![index, callback, sign_out, me, make_playlist])
        .attach(Template::fairing())
        .launch(); 
}

// fn rocket() -> rocket::Rocket {
//     rocket::ignite().
//         mount("/", routes![index, callback, sign_out, me, playlist, create_playlist])
// }

// #[cfg(test)]
// mod test {
//     use super::rocket;
//     use rocket::local::Client;
//     use rocket::http::Status;
//     use ureq::{ json, Error };

//     #[test]
//     fn make_playlist() {
//         // let client = Client::new(rocket()).expect("valid rocket instance");
//         let response = ureq::post("http://localhost:8000/me/playlist")
//             .set("Content-Type", "application/json")
//             .send_json(json!({
//                 "playlist": "test"
//             }));

//         match response {
//             Ok(response) => {
//                 println!("it works?: {:?}", response)
//             }
//             Err(Error::Status(code, response)) => {
//                 println!("Code: {}\nResponse: {:?}", code, response);
//             }
//             Err(_) => {
//                 println!(":((((")
//             }
//         }
//         assert_eq!(response.unwrap().status(), Status::Ok);
//     }
// }