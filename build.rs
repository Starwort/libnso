use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, COOKIE, DNT, REFERER, UPGRADE_INSECURE_REQUESTS};
fn main() {
    let find_script = Regex::new(r#"<script (?:defer(?:="?defer"?)? )?src="(/static/js/main\.[0-9a-fA-F]{8}\.js)">"#).unwrap();
    let client = Client::new();
    let page_text = client
        .get("https://api.lp1.av5ja.srv.nintendo.net")
        .header(UPGRADE_INSECURE_REQUESTS, "1")
        .header(ACCEPT, "*/*")
        .header(DNT, "1")
        .header("X-AppColorScheme", "DARK")
        .header("X-Requested-With", "com.nintendo.znca")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-User", "?1")
        .header("Sec-Fetch-Dest", "document")
        .header(COOKIE, "_dnt=1")
        .send()
        .expect("Nintendo API request failed")
        .text()
        .expect("Failed to read response body");
    let script_url = format!(
        "https://api.lp1.av5ja.srv.nintendo.net{}",
        find_script
            .captures(&page_text)
            .expect(&page_text)
            .get(1)
            .expect("Regex capture missing")
            .as_str()
    );
    println!("{script_url}");
    let script = client
        .get(script_url)
        .header(ACCEPT, "*/*")
        .header("X-Requested-With", "com.nintendo.znca")
        .header("Sec-Fetch-Site", "same-origin")
        .header("Sec-Fetch-Mode", "no-cors")
        .header("Sec-Fetch-Dest", "script")
        .header(REFERER, "https://api.lp1.av5ja.srv.nintendo.net/")
        .send()
        .expect("Nintendo API request failed")
        .text()
        .expect("Failed to read response body");
    let revision_pattern = Regex::new(concat!(
        r"\b(?P<revision>[0-9a-f]{40})",
        r#"\b[\S]*?void 0[\S]*?"revision_info_not_set"\}"#,
        r"`,.*?=`(?P<version>\d+\.\d+\.\d+)-",
    ))
    .unwrap();
    let revision_match = revision_pattern
        .captures(&script)
        .expect("Failed to find revision info");
    let version = revision_match
        .name("version")
        .expect("Regex capture 'version' missing")
        .as_str();
    let revision = revision_match
        .name("revision")
        .expect("Regex capture 'revision' missing")
        .as_str();
    println!(
        "cargo:rustc-env=SPLATOON3_WEB_VIEW_VERSION={version}-{}",
        &revision[..8]
    );
}
