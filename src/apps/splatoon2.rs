use reqwest::header::{
    ACCEPT,
    ACCEPT_ENCODING,
    ACCEPT_LANGUAGE,
    CONNECTION,
    DNT,
    HOST,
    USER_AGENT,
};
use reqwest::Client;

use crate::{get_game_web_token, AppFToken, WEB_VIEW_USER_AGENT};
/// Splatoon 2's internal ID
pub const GAME_ID: u64 = 5_741_031_244_955_648;

/// Get the Splatoon 2 access token, based on the user's F token and login token
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

/// Get the `iksm_session`, based on the user's Splatoon 2 access token
///
/// # Errors
///
/// An `Err` will be returned if the request to Nintendo fails (for example, if
/// the connection fails). If the server's response does not include the
/// `iksm_session`, `Ok(None)` will be returned.
pub async fn get_iksm_session(
    web_token: &str,
    client: &Client,
) -> Result<Option<String>, reqwest::Error> {
    Ok(client
        .get("https://app.splatoon2.nintendo.net/?lang=en-US")
        .header(HOST, "app.splatoon2.nintendo.net")
        .header("X-IsAppAnalyticsOptedIn", "false")
        .header(
            ACCEPT,
            concat!(
                "text/html,",
                "application/xhtml+xml,",
                "application/xml;q=0.9,",
                "*/*;q=0.8",
            ),
        )
        .header(ACCEPT_ENCODING, "gzip,deflate")
        .header("X-GameWebToken", web_token)
        .header(ACCEPT_LANGUAGE, "en-US")
        .header("X-IsAnalyticsOptedIn", "false")
        .header(CONNECTION, "keep-alive")
        .header(DNT, "0")
        .header(USER_AGENT, WEB_VIEW_USER_AGENT)
        .header("X-Requested-With", "com.nintendo.znca")
        .send()
        .await?
        .cookies()
        .find(|cookie| cookie.name() == "iksm_session")
        .map(|cookie| cookie.value().to_string()))
}
