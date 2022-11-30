use std::collections::HashMap;

use reqwest::header::{ACCEPT, ACCEPT_ENCODING, CONNECTION, HOST, USER_AGENT};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AppFToken, NSO_USER_AGENT, NSO_VERSION};

/// Get the access token for a game, based on the user's F token and login token
///
/// # Errors
///
/// If the request to Nintendo fails (for example, if a token provided is
/// invalid)
pub async fn get_game_web_token<const GAME_ID: u64>(
    f: &AppFToken,
    login_token: &str,
    client: &Client,
) -> Result<String, reqwest::Error> {
    #[derive(Serialize)]
    struct Body<'a> {
        parameter: Parameter<'a>,
    }
    #[derive(Serialize)]
    #[allow(non_snake_case)]
    struct Parameter<'a> {
        id: u64,
        f: &'a str,
        registrationToken: &'a str,
        timestamp: i64,
        requestId: &'a str,
    }
    #[derive(Deserialize)]
    struct Resp {
        result: Result,
        #[serde(flatten)]
        _rest: HashMap<String, Value>,
    }
    #[derive(Deserialize)]
    #[allow(non_snake_case)]
    struct Result {
        accessToken: String,
        #[serde(flatten)]
        _rest: HashMap<String, Value>,
    }
    Ok(client
        .post("https://api-lp1.znc.srv.nintendo.net/v2/Game/GetWebServiceToken")
        .header(HOST, "api-lp1.znc.srv.nintendo.net")
        .header(USER_AGENT, NSO_USER_AGENT)
        .header(ACCEPT, "application/json")
        .header("X-ProductVersion", NSO_VERSION)
        .header(CONNECTION, "Keep-Alive")
        .bearer_auth(login_token)
        .header("X-Platform", "Android")
        .header(ACCEPT_ENCODING, "gzip")
        .json(&Body {
            parameter: Parameter {
                id: GAME_ID,
                f: &f.f,
                registrationToken: login_token,
                timestamp: f.timestamp,
                requestId: &f.request_id,
            },
        })
        .send()
        .await?
        .json::<Resp>()
        .await?
        .result
        .accessToken)
}
