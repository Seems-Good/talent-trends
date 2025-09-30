use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

const OAUTH_TOKEN_URL: &str = "https://www.warcraftlogs.com/oauth/token";
const GRAPHQL_ENDPOINT: &str = "https://www.warcraftlogs.com/api/v2/client";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TalentData {
    pub name: String,
    pub talent_string: String,
    pub log_url: String,
}

#[derive(Debug, Clone)]
pub struct TalentDataWithRank {
    pub rank: usize,
    pub data: TalentData,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
  //  expires_in: u64,
}

lazy_static::lazy_static! {
    static ref TOKEN_CACHE: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
}

async fn get_access_token() -> Result<String> {
    {
        let cache = TOKEN_CACHE.read().await;
        if let Some(token) = cache.as_ref() {
            return Ok(token.clone());
        }
    }
    
    let client_id = std::env::var("WCL_CLIENT_ID")
        .context("WCL_CLIENT_ID not set in .env")?;
    let client_secret = std::env::var("WCL_CLIENT_SECRET")
        .context("WCL_CLIENT_SECRET not set in .env")?;
    
    tracing::info!("Fetching new OAuth token...");
    
    let client = Client::new();
    let params = [("grant_type", "client_credentials")];
    
    let response = client
        .post(OAUTH_TOKEN_URL)
        .basic_auth(client_id, Some(client_secret))
        .form(&params)
        .send()
        .await
        .context("Failed to request OAuth token")?;
    
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("OAuth failed with status {}: {}", status, error_text);
    }
    
    let token_resp: TokenResponse = response.json().await
        .context("Failed to parse OAuth token response")?;
    
    tracing::info!("✓ OAuth token acquired");
    
    {
        let mut cache = TOKEN_CACHE.write().await;
        *cache = Some(token_resp.access_token.clone());
    }
    
    Ok(token_resp.access_token)
}

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
}

async fn fetch_talent_string(
    client: &Client,
    token: &str,
    report_code: &str,
    fight_id: i64,
    player_name: &str,
) -> Result<String> {
    let query = r#"
    query GetTalents($reportCode: String!, $fightIDs: [Int]!) {
      reportData {
        report(code: $reportCode) {
          masterData(translate: true) {
            actors(type: "Player") {
              id
              name
            }
          }
          fights(fightIDs: $fightIDs) {
            id
          }
        }
      }
    }
    "#;
    
    let request = GraphQLRequest {
        query: query.to_string(),
        variables: Some(serde_json::json!({
            "reportCode": report_code,
            "fightIDs": [fight_id],
        })),
    };
    
    let response = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(token)
        .json(&request)
        .send()
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    
    let actors = json
        .pointer("/data/reportData/report/masterData/actors")
        .and_then(|v| v.as_array())
        .context("No actors found")?;
    
    let actor_id = actors
        .iter()
        .find(|a| {
            a.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n == player_name)
                .unwrap_or(false)
        })
        .and_then(|a| a.get("id"))
        .and_then(|id| id.as_i64())
        .context(format!("Actor {} not found in masterData", player_name))?;
    
    tracing::debug!("Found actor {} with ID {}", player_name, actor_id);
    
    let talent_query = r#"
    query GetTalentCode($reportCode: String!, $fightIDs: [Int]!, $actorID: Int!) {
      reportData {
        report(code: $reportCode) {
          fights(fightIDs: $fightIDs) {
            talentImportCode(actorID: $actorID)
          }
        }
      }
    }
    "#;
    
    let talent_request = GraphQLRequest {
        query: talent_query.to_string(),
        variables: Some(serde_json::json!({
            "reportCode": report_code,
            "fightIDs": [fight_id],
            "actorID": actor_id,
        })),
    };
    
    let talent_response = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(token)
        .json(&talent_request)
        .send()
        .await?;
    
    let talent_json: serde_json::Value = talent_response.json().await?;
    
    let talent_code = talent_json
        .pointer("/data/reportData/report/fights/0/talentImportCode")
        .and_then(|v| v.as_str())
        .context("No talent code found")?;
    
    Ok(talent_code.to_string())
}

pub async fn fetch_top_talents_stream(
    class: &str,
    spec: &str,
    encounter_id: i32,
    region: Option<&str>,
) -> Result<mpsc::Receiver<Result<TalentDataWithRank>>> {
    let (tx, rx) = mpsc::channel(10);
    
    let class = class.to_string();
    let spec = spec.to_string();
    let region = region.map(|s| s.to_string());
    
    tokio::spawn(async move {
        if let Err(e) = fetch_and_stream_talents(&tx, &class, &spec, encounter_id, region.as_deref()).await {
            let _ = tx.send(Err(e)).await;
        }
    });
    
    Ok(rx)
}

async fn fetch_and_stream_talents(
    tx: &mpsc::Sender<Result<TalentDataWithRank>>,
    class: &str,
    spec: &str,
    encounter_id: i32,
    region: Option<&str>,
) -> Result<()> {
    let token = get_access_token().await?;
    let client = Client::new();
    
    let class_name = class.replace('_', "");
    let region_display = region.unwrap_or("all");
    
    tracing::info!("Querying: class={}, spec={}, encounter={}, region={}", 
        class_name, spec, encounter_id, region_display);
    
    let query = r#"
    query Rankings($encounterId: Int!, $className: String!, $specName: String!, $serverRegion: String) {
      worldData {
        encounter(id: $encounterId) {
          name
          characterRankings(
            className: $className
            specName: $specName
            serverRegion: $serverRegion
            metric: dps
            difficulty: 5
            page: 1
          )
        }
      }
    }
    "#;
    
    let mut variables = serde_json::json!({
        "encounterId": encounter_id,
        "className": class_name,
        "specName": spec,
    });
    
    if let Some(r) = region {
        variables["serverRegion"] = serde_json::Value::String(r.to_string());
    }
    
    let request = GraphQLRequest {
        query: query.to_string(),
        variables: Some(variables),
    };
    
    let response = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(&token)
        .json(&request)
        .send()
        .await?;
    
    let status = response.status();
    let response_text = response.text().await?;
    
    if !status.is_success() {
        anyhow::bail!("GraphQL request failed with status {}: {}", status, response_text);
    }
    
    let json: serde_json::Value = serde_json::from_str(&response_text)?;
    
    if let Some(errors) = json.get("errors") {
        anyhow::bail!("GraphQL errors: {}", serde_json::to_string_pretty(errors)?);
    }
    
    let rankings = json
        .pointer("/data/worldData/encounter/characterRankings/rankings")
        .and_then(|v| v.as_array())
        .context("No rankings found in response")?;
    
    tracing::info!("Found {} rankings, streaming results...", rankings.len());
    
    let mut rank_number = 1;
    
    for rank in rankings.iter() {
        if rank_number > 10 {
            break;
        }
        
        let name = rank.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
        
        if name == "Anonymous" {
            tracing::debug!("Skipping Anonymous log");
            continue;
        }
        
        let report_code = rank.pointer("/report/code").and_then(|v| v.as_str()).unwrap_or("");
        let fight_id = rank.pointer("/report/fightID").and_then(|v| v.as_i64()).unwrap_or(0);
        
        let log_url = format!("https://www.warcraftlogs.com/reports/{}#fight={}", report_code, fight_id);
        
        let talent_string = if !report_code.is_empty() && fight_id > 0 {
            match fetch_talent_string(&client, &token, report_code, fight_id, name).await {
                Ok(s) if !s.is_empty() => s,
                Err(e) => {
                    tracing::warn!("Failed to fetch talents for {}: {:#}", name, e);
                    "[Talent data unavailable]".to_string()
                }
                _ => "[Talent data unavailable]".to_string(),
            }
        } else {
            "[Missing report data]".to_string()
        };
        
        tracing::info!("✓ Rank {}: {}", rank_number, name);
        
        let talent_data = TalentDataWithRank {
            rank: rank_number,
            data: TalentData {
                name: name.to_string(),
                talent_string,
                log_url,
            },
        };
        
        if tx.send(Ok(talent_data)).await.is_err() {
            break;
        }
        
        rank_number += 1;
    }
    
    Ok(())
}
