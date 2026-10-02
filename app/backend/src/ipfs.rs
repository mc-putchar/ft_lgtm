use crate::models::RunResult;

use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use tracing::instrument;

#[derive(Deserialize)]
struct IpfsAddResponse {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Hash")]
    hash: String,
}

#[instrument(name = "publish_to_ipfs", skip_all)]
pub async fn publish_to_ipfs(code: &str, output: &RunResult) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let output_json = serde_json::to_string(output).unwrap_or_default();

    let form = Form::new()
        .part(
            "source",
            Part::text(code.to_string()).file_name("source.txt"),
        )
        .part("output", Part::text(output_json).file_name("output.json"));

    let base_url = std::env::var("IPFS_API_URL").unwrap_or_else(|_| "http://ipfs:5001".to_string());
    let response = client
        .post(format!("{}/api/v0/add?wrap-with-directory=true", base_url))
        .multipart(form)
        .send()
        .await?
        .text()
        .await?;

    let mut dir_cid = String::new();
    for line in response.lines() {
        if let Ok(parsed) = serde_json::from_str::<IpfsAddResponse>(line) {
            if parsed.name == "" {
                dir_cid = parsed.hash;
            }
        }
    }

    Ok(dir_cid)
}

pub async fn fetch_from_ipfs(cid: &str) -> Result<RunResult, reqwest::Error> {
    let client = reqwest::Client::new();

    let base_url = std::env::var("IPFS_API_URL").unwrap_or_else(|_| "http://ipfs:5001".to_string());
    let url = format!("{}/api/v0/cat?arg={}/output.json", base_url, cid);

    let response = client.post(&url).send().await?.text().await?;
    let data: RunResult = serde_json::from_str(&response).unwrap();

    Ok(data)
}
