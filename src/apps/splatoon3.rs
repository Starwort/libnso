use std::collections::HashMap;

use reqwest::header::{ACCEPT_LANGUAGE, COOKIE, ORIGIN, REFERER, USER_AGENT};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{get_game_web_token, AppFToken, WEB_VIEW_USER_AGENT};

/// Version of the Splatoon 3 API being mocked
pub const WEB_VIEW_VERSION: &str = env!("SPLATOON3_WEB_VIEW_VERSION");
/// Splatoon 3's internal ID
pub const GAME_ID: u64 = 4_834_290_508_791_808;

#[doc(hidden)]
pub mod keys {
    /// `StageScheduleQuery`
    pub const SCHEDULES: &str = "7d4bb0565342b7385ceb97d109e14897";
    /// `GesotownQuery`
    pub const SPLATNET: &str = "a43dd44899a09013bcfd29b4b13314ff";
    /// `CoopHistoryQuery`
    pub const SALMON: &str = "817618ce39bcf5570f52a97d73301b30";
    /// `SaleGearDetailOrderGesotownGearMutation`
    pub const ORDER: &str = "b79b7a101a243912754f72437e2ad7e5";
    /// `FestRecordQuery`
    pub const SPLATFEST_OVERVIEW: &str = "44c76790b68ca0f3da87f2a3452de986";
    /// `DetailFestRecordDetailQuery`
    pub const SPLATFEST: &str = "2d661988c055d843b3be290f04fb0db9";
    /// `LatestBattleHistoriesQuery`,
    pub const LATEST_BATTLES: &str = "7d8b560e31617e981cf7c8aa1ca13a00";
    /// `MyOutfitCommonDataEquipmentsQuery`
    pub const GEAR: &str = "d29cd0c2b5e6bac90dd5b817914832f8";
}

/// Get the Splatoon 3 access token, based on the user's F token and login token
///
/// # Errors
///
/// If the request to Nintendo fails (for example, if a token provided is
/// invalid)
pub async fn get_web_token(
    f: &AppFToken,
    login_token: &str,
    client: &Client,
) -> Result<String, reqwest::Error> {
    get_game_web_token::<GAME_ID>(f, login_token, client).await
}

/// Get the `bullet_token`, based on the user's Splatoon 2 access token
///
/// # Errors
///
/// If the request to Nintendo fails (for example, if a token provided is
/// invalid)
pub async fn get_bullet_token(
    web_token: &str,
    client: &Client,
) -> Result<String, reqwest::Error> {
    #[derive(Deserialize)]
    #[allow(non_snake_case)]
    struct Resp {
        bulletToken: String,
        #[serde(flatten)]
        _rest: HashMap<String, Value>,
    }
    Ok(client
        .post("https://api.lp1.av5ja.srv.nintendo.net/api/bullet_tokens")
        .header(ORIGIN, "https://api.lp1.av5ja.srv.nintendo.net")
        .header(REFERER, "https://api.lp1.av5ja.srv.nintendo.net/")
        .header("X-Web-View-Ver", WEB_VIEW_VERSION)
        .header(USER_AGENT, WEB_VIEW_USER_AGENT)
        .header(COOKIE, format!("_dnt=0;_gtoken={web_token}"))
        .send()
        .await?
        .json::<Resp>()
        .await?
        .bulletToken)
}

#[doc(hidden)]
pub async fn graphql_query_with_variables<T: Serialize>(
    bullet_token: &str,
    lang: &str,
    web_token: &str,
    query_hash: &'static str,
    variables: T,
    client: &Client,
) -> Result<Response, reqwest::Error> {
    #[derive(Serialize)]
    struct Body<T> {
        extensions: Extensions,
        variables: T,
    }
    #[derive(Serialize)]
    #[allow(non_snake_case)]
    struct Extensions {
        persistedQuery: PersistedQuery,
    }
    #[derive(Serialize)]
    #[allow(non_snake_case)]
    struct PersistedQuery {
        sha256Hash: &'static str,
        version: u32,
    }
    client
        .post("https://api.lp1.av5ja.srv.nintendo.net/api/graphql")
        .bearer_auth(bullet_token)
        .header("X-Web-View-Ver", WEB_VIEW_VERSION)
        .header(ORIGIN, "https://api.lp1.av5ja.srv.nintendo.net")
        .header(
            REFERER,
            "https://api.lp1.av5ja.srv.nintendo.net/schedule/regular",
        )
        .header(USER_AGENT, WEB_VIEW_USER_AGENT)
        .header(ACCEPT_LANGUAGE, lang)
        .header("X-Requested-With", "XMLHttpRequest")
        .header(COOKIE, format!("_dnt=0;_gtoken={web_token}"))
        .json(&Body {
            extensions: Extensions {
                persistedQuery: PersistedQuery {
                    sha256Hash: query_hash,
                    version: 1,
                },
            },
            variables,
        })
        .send()
        .await
}

#[doc(hidden)]
pub async fn graphql_query(
    bullet_token: &str,
    lang: &str,
    web_token: &str,
    query_hash: &'static str,
    client: &Client,
) -> Result<Response, reqwest::Error> {
    #[derive(Serialize)]
    struct NoVariables {}
    graphql_query_with_variables(
        bullet_token,
        lang,
        web_token,
        query_hash,
        NoVariables {},
        client,
    )
    .await
}
