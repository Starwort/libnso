use std::io::{stdin, stdout, Write};

use nso::splatoon3::{self, graphql_query, keys};
use nso::{
    get_access_token,
    get_f1,
    get_f2,
    get_login_token,
    get_login_url_and_verifier,
    get_session_token,
    get_session_token_code_from_select_url,
    get_user_info,
    UrlAndVerifier,
};
use reqwest::Client;

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() {
    let UrlAndVerifier {
        url,
        verifier,
    } = get_login_url_and_verifier();
    println!("Login URL: {url}");
    println!("Enter the 'Select this person' URL here:");
    print!(">>> ");
    stdout().flush().expect("Unexpected IO error");
    let select_url = stdin()
        .lines()
        .next()
        .expect("Unexpected EOF")
        .expect("Unexpected IO error");
    let session_token_code = get_session_token_code_from_select_url(select_url.trim())
        .expect("Invalid 'Select this person' URL");
    println!("session_token_code: {session_token_code}");
    let client = Client::new();
    let session_token = get_session_token(session_token_code, &verifier, &client)
        .await
        .expect("Failed to get session_token");
    println!("session_token: {session_token}");
    let tokens = get_access_token(&session_token, &client)
        .await
        .expect("Failed to get tokens");
    println!("access_token: {}", tokens.access_token);
    println!("id_token: {}", tokens.id_token);
    let user_info = get_user_info(&tokens.access_token, &client)
        .await
        .expect("Failed to get user_info");
    let nso_f = get_f1(&tokens.id_token, &client)
        .await
        .expect("Failed to get nso_f");
    let login_token = get_login_token(&nso_f, &tokens.id_token, &user_info, &client)
        .await
        .expect("Failed to get login_token");
    let app_f = get_f2(&login_token, &client)
        .await
        .expect("Failed to get app_f");
    let web_token = splatoon3::get_web_token(&app_f, &login_token, &client)
        .await
        .expect("Failed to get web_token");
    let bullet_token = splatoon3::get_bullet_token(&web_token, &client)
        .await
        .expect("Failed to get bullet_token");
    println!("web_token: {web_token}");
    println!("bullet_token: {bullet_token}");
    println!("---");
    println!(
        "Schedules: {}",
        graphql_query(
            &bullet_token,
            &user_info.language,
            &web_token,
            keys::SCHEDULES,
            &client,
        )
        .await
        .expect("GraphQL query failed unexpectedly")
        .text()
        .await
        .expect("GraphQL query failed")
    );
    println!("---");
    println!(
        "Splatnet: {}",
        graphql_query(
            &bullet_token,
            &user_info.language,
            &web_token,
            keys::SPLATNET,
            &client,
        )
        .await
        .expect("GraphQL query failed unexpectedly")
        .text()
        .await
        .expect("GraphQL query failed")
    );
    println!("---");
    println!(
        "Salmon: {}",
        graphql_query(
            &bullet_token,
            &user_info.language,
            &web_token,
            keys::SALMON,
            &client,
        )
        .await
        .expect("GraphQL query failed unexpectedly")
        .text()
        .await
        .expect("GraphQL query failed")
    );
    println!("---");
    println!(
        "Splatfest overview: {}",
        graphql_query(
            &bullet_token,
            &user_info.language,
            &web_token,
            keys::SPLATFEST_OVERVIEW,
            &client,
        )
        .await
        .expect("GraphQL query failed unexpectedly")
        .text()
        .await
        .expect("GraphQL query failed")
    );
    println!("---");
    println!(
        "Latest battles: {}",
        graphql_query(
            &bullet_token,
            &user_info.language,
            &web_token,
            keys::LATEST_BATTLES,
            &client,
        )
        .await
        .expect("GraphQL query failed unexpectedly")
        .text()
        .await
        .expect("GraphQL query failed")
    );
    println!("---");
    println!(
        "Gear: {}",
        graphql_query(
            &bullet_token,
            &user_info.language,
            &web_token,
            keys::GEAR,
            &client,
        )
        .await
        .expect("GraphQL query failed unexpectedly")
        .text()
        .await
        .expect("GraphQL query failed")
    );
}
