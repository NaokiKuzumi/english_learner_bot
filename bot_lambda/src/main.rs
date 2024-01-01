use std::collections::HashMap;
use aws_config::BehaviorVersion;
use log::LevelFilter;
use reqwest::{Client};
use serde::{Deserialize, Serialize};
use lambda_runtime::{LambdaEvent, service_fn};
use rand::{Rng, thread_rng};
use scraper::{Html, Selector};
use serde_json::Value;

type LambdaError = Box<dyn std::error::Error + Send + Sync + 'static>;


#[derive(Debug, Serialize, Deserialize)]
struct SecretConfig {
    username: String,
    password: String,
}

impl Default for SecretConfig {
    fn default() -> Self {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OAuthRequest {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    grant_type: String,
    username: String,
    password: String,
    scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OAuthResponse {
    access_token: String,
    token_type: String,
    scope: String,
    created_at: u64,
}

/// Most of the field type just from guessing. This app only cares about status.
#[derive(Debug, Serialize, Deserialize)]
struct PostData {
    status: String,
    in_reply_to_id: Option<String>,
    quote_id: Option<String>,
    media_ids: Vec<String>,
    sensitive: bool,
    spoiler_text: String,
    visibility: String,
    content_type: String,
    poll: Option<HashMap<String, String>>,
    scheduled_at: Option<String>,
    to: Vec<String>,
}

/// Returns the oauth access token.
/// It seems like to live so long, so I don't care about refreshing for now.
async fn truth_access_token(username: &str, password: &str) -> anyhow::Result<String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:122.0) Gecko/20100101 Firefox/122.0")
        .cookie_store(true)
        .gzip(true)
        .use_rustls_tls()
        .build()?;

    // Tested for cookie, but turned out it doesn't need cookie.
    // client.get("https://truthsocial.com/")
    //     .send().await?;
    // async_std::task::sleep(Duration::from_secs(1)).await;

    let o_auth_request = OAuthRequest {
        // ID/secrets exposed in web UI.
        // The /developers/apps/create endpoint registered one didn't work. f*ck.
        client_id: "9X1Fdd-pxNsAgEDNi_SfhJWi8T-vLuV2WVzKIbkTCw4".to_string(),
        client_secret: "ozF8jzI4968oTKFkEnsBC-UbLPCdrSv0MkXGQu2o_-M".to_string(),
        redirect_uri: "urn:ietf:wg:oauth:2.0:oob".to_string(),
        grant_type: "password".to_string(),
        username: username.to_string(),
        password: password.to_string(),
        scope: "read write follow push".to_string(),
    };
    let str = serde_json::to_string(&o_auth_request)?;

    let response = client.post("https://truthsocial.com/oauth/token")
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-encoding", "gzip")
        .header("Referer", "https://truthsocial.com/login")
        .header("Host", "truthsocial.com")
        .header("Content-Type", "application/json")
        .header("Origin", "https://truthsocial.com")
        .header("Content-Length", str.bytes().len())
        .json(&o_auth_request)
        .send().await?;

    let result = response.json::<OAuthResponse>().await?;
    println!("{:?}", result);

    Ok(result.access_token)
}

async fn post_truth(client: &Client, token: &str, truth: &str) -> anyhow::Result<()> {
    let post_data = PostData {
        status: truth.to_string(),
        in_reply_to_id: None,
        quote_id: None,
        media_ids: vec![],
        sensitive: false,
        spoiler_text: "".to_string(),
        visibility: "public".to_string(),
        content_type: "text/plain".to_string(),
        poll: None,
        scheduled_at: None,
        to: vec![],
    };
    client.post("https://truthsocial.com/api/v1/statuses")
        .header("Authorization", format!("Bearer {}", token))
        .json(&post_data)
        .send().await?;

    Ok(())
}


async fn get_meaning(client: &Client, word: &str) -> anyhow::Result<String> {
    let doc = client.get(format!("https://ejje.weblio.jp/content/{}", word))
        .send().await?
        .text().await?;
    let doc = Html::parse_document(&doc);
    let selector = Selector::parse("span.content-explanation").unwrap();
    let meaning = doc.select(&selector).next().unwrap().text().collect::<String>();


    Ok(meaning.trim().to_owned())
}

async fn post_one_word_truth(client: &Client, token: &str, word_data: &str, data_size: u32) -> anyhow::Result<()> {
    let mut rng = thread_rng();
    let l = rng.gen_range(0..data_size);
    let word = word_data.split("\n").nth(l as usize).unwrap();
    let meaning = get_meaning(client, word).await?;

    post_truth(client, token, &format!("Truth英単語 {}: {}\n\n{}", l + 1, word, meaning)).await
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    simple_logging::log_to_stderr(LevelFilter::Warn);

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_secretsmanager::Client::new(&config);
    let secrets = client.get_secret_value().secret_id("english_learner_bot_secret").send().await?;
    let secret_config = if let Some(content) = secrets.secret_string {
        serde_json::from_str(&content)?
    } else {
        SecretConfig::default()
    };

    let access_token = truth_access_token(&secret_config.username, &secret_config.password).await?;
    let access_token_ref = &access_token;

    // UTF-8 text of only 1 word in one line, no doublequote expected.
    let word_database = include_str!("../data/jev+hev.csv");
    let word_db_size = 3018;


    let func = service_fn(move |_event: LambdaEvent<Value>| async move {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:122.0) Gecko/20100101 Firefox/122.0")
            .cookie_store(true)
            .gzip(true)
            .use_rustls_tls()
            .build()?;
        post_one_word_truth(&client, access_token_ref, word_database, word_db_size).await?;

        Ok::<(), LambdaError>(())
    });
    lambda_runtime::run(func).await?;
    Ok::<(), LambdaError>(())
}

#[cfg(test)]
mod test {
    use reqwest::Client;
    use crate::get_meaning;

    #[test]
    fn meaning() {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:122.0) Gecko/20100101 Firefox/122.0")
            .use_rustls_tls()
            .build().unwrap();
        let actual = tokio_test::block_on(get_meaning(&client, "truth"));
        assert!(actual.is_ok());
        assert_eq!("真理、真、真実、真相、事実、本当のこと、真実性、(事の)真偽、誠実、正直", actual.ok().unwrap());
    }
}
