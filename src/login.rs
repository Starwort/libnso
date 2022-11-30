use std::collections::HashMap;

use base64::URL_SAFE;
use const_format::formatcp;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use reqwest::header::{
    ACCEPT,
    ACCEPT_ENCODING,
    ACCEPT_LANGUAGE,
    AUTHORIZATION,
    CONNECTION,
    HOST,
    USER_AGENT,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// The version of Nintendo Switch Online that this library was built to mimic
pub const NSO_VERSION: &str = "2.3.1";
/// The user agent used to mock Nintendo Switch Online calls
pub const NSO_USER_AGENT: &str =
    formatcp!("com.nintendo.znca/{NSO_VERSION}, (Android/7.1.2)");
/// The user agent used to mock Nintendo Switch Online Lounge calls
pub const ONLINE_LOUNGE_USER_AGENT: &str =
    formatcp!("OnlineLounge/{NSO_VERSION} NASDKAPI Android");
/// The user agent used to mock webview calls
pub const WEB_VIEW_USER_AGENT: &str = concat!(
    "Mozilla/5.0 (Linux; Android 7.1.2; Pixel Build/NJH47D; wv) ",
    "AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 ",
    "Chrome/59.0.3071.125 Mobile ",
    "Safari/537.36"
);
/// The user agent used to communicate with non-Nintendo servers such as imink
pub const RUST_NSO_USER_AGENT: &str =
    formatcp!("rust-nso/{}", env!("CARGO_PKG_VERSION"));

pub struct UrlAndVerifier {
    pub url: String,
    pub verifier: String,
}

/// Generates a login URL and verifier code.
#[must_use]
pub fn get_login_url_and_verifier() -> UrlAndVerifier {
    let mut rng = ChaChaRng::from_entropy();
    let auth_state = base64::encode_config(
        {
            let mut data = [0u8; 36];
            rng.fill(&mut data[..32]);
            rng.fill(&mut data[32..]);
            data
        },
        URL_SAFE,
    )
    .trim_end_matches('=')
    .to_string();

    let auth_code_verifier = base64::encode_config(
        {
            let mut data = [0u8; 32];
            rng.fill(&mut data);
            data
        },
        URL_SAFE,
    )
    .trim_end_matches('=')
    .to_string();

    let acv_hash = Sha256::digest(&auth_code_verifier);
    let auth_code_challenge = base64::encode_config(acv_hash.as_slice(), URL_SAFE)
        .trim_end_matches('=')
        .to_string();

    UrlAndVerifier {
        url: format!(
            concat!(
                "https://accounts.nintendo.com/connect/1.0.0/authorize",
                "?state={}",
                "&redirect_uri=npf71b963c1b7b6d119://auth",
                "&client_id=71b963c1b7b6d119",
                "&scope=openid+user+user.birthday+user.mii+user.screenName",
                "&response_type=session_token_code",
                "&session_token_code_challenge={}",
                "&session_token_code_challenge_method=S256",
                "&theme=login_form",
            ),
            auth_state, auth_code_challenge,
        ),
        verifier: auth_code_verifier,
    }
}

/// Extract the user's `session_token_code` from their 'Select this person' URL
///
/// Returns `None` if the URL is invalid.
#[must_use]
pub fn get_session_token_code_from_select_url(select_url: &str) -> Option<&str> {
    select_url
        .strip_prefix("npf71b963c1b7b6d119://auth#")?
        .split('&')
        .find_map(|segment| segment.strip_prefix("session_token_code="))
}

/// Get `session_token` from Nintendo, based on `session_token_code` and
/// `auth_code_verifier`
///
/// # Errors
///
/// This function will fail if the request to Nintendo's servers fails.
pub async fn get_session_token(
    session_token_code: &str,
    auth_code_verifier: &str,
    client: &Client,
) -> Result<String, reqwest::Error> {
    #[derive(Serialize)]
    struct Body<'a> {
        client_id: &'static str,
        session_token_code: &'a str,
        session_token_code_verifier: &'a str,
    }
    impl<'a> Body<'a> {
        fn new(code: &'a str, verifier: &'a str) -> Self {
            Self {
                client_id: "71b963c1b7b6d119",
                session_token_code: code,
                session_token_code_verifier: verifier,
            }
        }
    }
    #[derive(Debug, Deserialize)]
    struct Resp {
        session_token: String,
        #[serde(flatten)]
        _rest: HashMap<String, Value>,
    }
    Ok(client
        .post("https://accounts.nintendo.com/connect/1.0.0/api/session_token")
        .header(USER_AGENT, ONLINE_LOUNGE_USER_AGENT)
        .header(ACCEPT_LANGUAGE, "en-US")
        .header(ACCEPT, "application/json")
        .header(HOST, "accounts.nintendo.com")
        .header(CONNECTION, "Keep-Alive")
        .header(ACCEPT_ENCODING, "gzip")
        .form(&Body::new(session_token_code, auth_code_verifier))
        .send()
        .await?
        .json::<Resp>()
        .await?
        .session_token)
}

#[derive(Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub id_token: String,
    pub expires_in: u64,
    /// Should always be `"Bearer"`, but this isn't checked
    pub token_type: String,
    /// Should always be
    /// `["openid", "user", "user.birthday", "user.mii", "user.screenName"]`,
    /// but this isn't checked
    pub scope: [String; 5],
}

/// Get the access and ID tokens from Nintendo, based on a `session_token`
///
/// # Errors
///
/// This function will fail if the request to Nintendo's servers fails.
pub async fn get_access_token(
    session_token: &str,
    client: &Client,
) -> Result<Tokens, reqwest::Error> {
    #[derive(Serialize)]
    struct Body<'a> {
        client_id: &'static str,
        session_token: &'a str,
        grant_type: &'static str,
    }
    impl<'a> Body<'a> {
        fn new(session_token: &'a str) -> Self {
            Self {
                client_id: "71b963c1b7b6d119",
                session_token,
                grant_type: "urn:ietf:params:oauth:grant-type:jwt-bearer-session-token",
            }
        }
    }

    client
        .post("https://accounts.nintendo.com/connect/1.0.0/api/token")
        .header(USER_AGENT, ONLINE_LOUNGE_USER_AGENT)
        .header(ACCEPT_LANGUAGE, "en-US")
        .header(ACCEPT, "application/json")
        .header(HOST, "accounts.nintendo.com")
        .header(CONNECTION, "Keep-Alive")
        .header(ACCEPT_ENCODING, "gzip")
        .json(&Body::new(session_token))
        .send()
        .await?
        .json()
        .await
}

#[derive(Deserialize)]
pub struct UserInfo {
    pub country: String,
    pub birthday: String,
    pub language: String,
    #[serde(flatten)]
    rest: HashMap<String, Value>,
    // timezone: {id: String, name: String, utcOffset: String, utcOffsetSeconds: i64},
    // clientFriendsOptedIn: bool,
    // analyticsPermissions: {
    //     internalAnalysis: {permitted: bool, updatedAt: u64},
    //     targetMarketing: {permitted: bool, updatedAt: u64}
    // },
    // id: String,
    // analyticsOptedIn: bool,
    // isChild: bool,
    // mii: {
    //     clientId: String,
    //     coreData: {"4": String},
    //     etag: String,
    //     favoriteColor: String,
    //     id: String,
    //     imageOrigin: String,
    //     imageUriTemplate: String,
    //     storeData: {"3": String},
    //     type: String,
    //     updatedAt: u64,
    // },
    // eachEmailOptedIn: {
    //     deals: {optedIn: bool, updatedAt: u64},
    //     survey: {optedIn: bool, updatedAt: u64}
    // },
    // screenName: String,
    // analyticsOptedInUpdatedAt: u64,
    // emailOptedIn: bool,
    // createdAt: u64
    // emailVerified: bool,
    // region: (no idea, mine was null)
    // clientFriendsOptedInUpdatedAt: u64,
    // gender: String,
    // candidateMiis: Vec<(no idea, mine was empty)>,
    // emailOptedInUpdatedAt: u64,
    // nickname: String,
    // updatedAt: u64
}

/// Get a user's country, birthday, and language
///
/// # Errors
///
/// This function will fail if the request to Nintendo's servers fails.
pub async fn get_user_info(
    access_token: &str,
    client: &Client,
) -> Result<UserInfo, reqwest::Error> {
    let mut rv: UserInfo = client
        .get("https://api.accounts.nintendo.com/2.0.0/users/me")
        .header(USER_AGENT, ONLINE_LOUNGE_USER_AGENT)
        .header(ACCEPT_LANGUAGE, "en-US")
        .header(ACCEPT, "application/json")
        .bearer_auth(access_token)
        .header(HOST, "api.accounts.nintendo.com")
        .header(CONNECTION, "Keep-Alive")
        .header(ACCEPT_ENCODING, "gzip")
        .send()
        .await?
        .json()
        .await?;
    rv.rest.clear();
    rv.rest.shrink_to_fit();
    Ok(rv)
}

#[derive(Deserialize)]
pub struct NsoFToken {
    pub f: String,
    pub timestamp: i64,
    pub request_id: String,
}

#[derive(Serialize)]
struct FTokenApiBody<'a> {
    hash_method: &'static str,
    token: &'a str,
}
impl<'a> FTokenApiBody<'a> {
    fn step_1(id_token: &'a str) -> Self {
        Self {
            hash_method: "1",
            token: id_token,
        }
    }

    fn step_2(login_token: &'a str) -> Self {
        Self {
            hash_method: "2",
            token: login_token,
        }
    }
}

/// Get an NSO f-token, from the provided `id_token`
///
/// # Errors
///
/// If the request to imink fails.
pub async fn get_f1(
    id_token: &str,
    client: &Client,
) -> Result<NsoFToken, reqwest::Error> {
    // TODO: Reverse-engineer libvoip so we don't depend on imink
    client
        .post("https://api.imink.app/f")
        .header(USER_AGENT, RUST_NSO_USER_AGENT)
        .json(&FTokenApiBody::step_1(id_token))
        .send()
        .await?
        .json()
        .await
}

/// Get a login token based on the first f-token
///
/// # Errors
///
/// This function will fail if the request to Nintendo's servers fails.
pub async fn get_login_token(
    f1: &NsoFToken,
    id_token: &str,
    user_info: &UserInfo,
    client: &Client,
) -> Result<String, reqwest::Error> {
    #[derive(Serialize)]
    struct Body<'a> {
        parameter: Parameter<'a>,
    }
    #[derive(Serialize)]
    #[allow(non_snake_case)]
    struct Parameter<'a> {
        f: &'a str,
        naIdToken: &'a str,
        timestamp: i64,
        requestId: &'a str,
        naCountry: &'a str,
        naBirthday: &'a str,
        language: &'a str,
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
        webApiServerCredential: Credential,
        #[serde(flatten)]
        _rest: HashMap<String, Value>,
    }
    #[derive(Deserialize)]
    #[allow(non_snake_case)]
    struct Credential {
        accessToken: String,
        #[serde(flatten)]
        _rest: HashMap<String, Value>,
    }

    Ok(client
        .post("https://api-lp1.znc.srv.nintendo.net/v3/Account/Login")
        .header(HOST, "api-lp1.znc.srv.nintendo.net")
        .header(ACCEPT_LANGUAGE, "en-US")
        .header(USER_AGENT, NSO_USER_AGENT)
        .header(ACCEPT, "application/json")
        .header("X-ProductVersion", NSO_VERSION)
        .header(CONNECTION, "Keep-Alive")
        .header(AUTHORIZATION, "Bearer")
        .header("X-Platform", "Android")
        .header(ACCEPT_ENCODING, "gzip")
        .json(&Body {
            parameter: Parameter {
                f: &f1.f,
                timestamp: f1.timestamp,
                requestId: &f1.request_id,
                naIdToken: id_token,
                naCountry: &user_info.country,
                naBirthday: &user_info.birthday,
                language: &user_info.language,
            },
        })
        .send()
        .await?
        .json::<Resp>()
        .await?
        .result
        .webApiServerCredential
        .accessToken)
}

#[derive(Deserialize)]
pub struct AppFToken {
    pub f: String,
    pub timestamp: i64,
    pub request_id: String,
}

/// Get an application f-token, from the provided `login_token`
///
/// # Errors
///
/// If the request to imink fails.
pub async fn get_f2(
    login_token: &str,
    client: &Client,
) -> Result<AppFToken, reqwest::Error> {
    // TODO: Reverse-engineer libvoip so we don't depend on imink
    client
        .post("https://api.imink.app/f")
        .header(USER_AGENT, RUST_NSO_USER_AGENT)
        .json(&FTokenApiBody::step_2(login_token))
        .send()
        .await?
        .json()
        .await
}
