use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Warcraft Logs Endpoints:
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

    let client_id = std::env::var("WCL_CLIENT_ID").context("WCL_CLIENT_ID not set in .env?")?;
    let client_secret =
        std::env::var("WCL_CLIENT_SECRET").context("WCL_CLIENT_SECRET not set in .env?")?;

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

    let token_resp: TokenResponse = response
        .json()
        .await
        .context("Failed to parse OAuth token response")?;

    tracing::info!("OAuth token acquired");

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

/// Two-step talent lookup:
/// 1. Resolve the player's actor ID from masterData
/// 2. Fetch talentImportCode from the fight using that actor ID
async fn fetch_talent_string(
    client: &Client,
    token: &str,
    report_code: &str,
    fight_id: i64,
    player_name: &str,
) -> Result<String> {
    // Step 1: resolve actor ID
    let actor_query = r#"
    query GetActors($reportCode: String!) {
      reportData {
        report(code: $reportCode) {
          masterData(translate: true) {
            actors(type: "Player") {
              id
              name
            }
          }
        }
      }
    }
    "#;

    let actor_request = GraphQLRequest {
        query: actor_query.to_string(),
        variables: Some(serde_json::json!({
            "reportCode": report_code,
        })),
    };

    let actor_json: serde_json::Value = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(token)
        .json(&actor_request)
        .send()
        .await
        .context("Failed to send actor lookup request")?
        .json()
        .await
        .context("Failed to parse actor lookup response")?;

    let actors = actor_json
        .pointer("/data/reportData/report/masterData/actors")
        .and_then(|v| v.as_array())
        .context("No actors array in masterData")?;

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
        .with_context(|| format!("Actor '{}' not found in masterData", player_name))?;

    tracing::debug!("Resolved actor '{}' -> ID {}", player_name, actor_id);

    // Step 2: fetch talentImportCode for that actor in that fight
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

    let talent_json: serde_json::Value = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(token)
        .json(&GraphQLRequest {
            query: talent_query.to_string(),
            variables: Some(serde_json::json!({
                "reportCode": report_code,
                "fightIDs": [fight_id as i32],
                "actorID": actor_id as i32,
            })),
        })
        .send()
        .await
        .context("Failed to send talent code request")?
        .json()
        .await
        .context("Failed to parse talent code response")?;

    let talent_code = talent_json
        .pointer("/data/reportData/report/fights/0/talentImportCode")
        .and_then(|v| v.as_str())
        .context("No talentImportCode in fight response")?;

    Ok(talent_code.to_string())
}

pub async fn fetch_top_talents_stream(
    class: &str,
    spec: &str,
    encounter_id: i32,
    region: Option<&str>,
    difficulty: i32,
    partition: Option<i32>,
) -> Result<mpsc::Receiver<Result<TalentDataWithRank>>> {
    let (tx, rx) = mpsc::channel(10);

    let class = class.to_string();
    let spec = spec.to_string();
    let region = region.map(|s| s.to_string());

    tokio::spawn(async move {
        if let Err(e) = fetch_and_stream_talents(
            &tx,
            &class,
            &spec,
            encounter_id,
            region.as_deref(),
            difficulty,
            partition,
        )
        .await
        {
            tracing::error!("fetch_and_stream_talents failed: {:#}", e);
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
    difficulty: i32,
    partition: Option<i32>,
) -> Result<()> {
    let token = get_access_token().await?;
    let client = Client::new();

    // WCL expects "DeathKnight" not "Death_Knight" — strip underscores
    let class_name = class.replace('_', "");
    let region_display = region.unwrap_or("all");

    tracing::info!(
        "Querying {} {} encounter {} region {} difficulty {} partition {:?}",
        class_name, spec, encounter_id, region_display, difficulty, partition
    );

    // Only inject the partition argument when one is configured for this season
    let partition_arg = match partition {
        Some(p) => format!("partition: {}", p),
        None => String::new(),
    };

    let query = format!(
        r#"
        query Rankings(
          $encounterId: Int!,
          $className: String!,
          $specName: String!,
          $serverRegion: String,
          $difficulty: Int!
        ) {{
          worldData {{
            encounter(id: $encounterId) {{
              name
              characterRankings(
                className: $className
                specName: $specName
                serverRegion: $serverRegion
                metric: dps
                difficulty: $difficulty
                page: 1
                {partition_arg}
              )
            }}
          }}
        }}
        "#,
        partition_arg = partition_arg
    );

    let mut variables = serde_json::json!({
        "encounterId": encounter_id,
        "className": class_name,
        "specName": spec,
        "difficulty": difficulty,
    });

    if let Some(r) = region {
        variables["serverRegion"] = serde_json::Value::String(r.to_string());
    }

    let response = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(&token)
        .json(&GraphQLRequest {
            query,
            variables: Some(variables),
        })
        .send()
        .await
        .context("Failed to send rankings GraphQL request")?;

    let status = response.status();
    let response_text = response.text().await?;

    if !status.is_success() {
        anyhow::bail!(
            "GraphQL request failed with status {}: {}",
            status,
            response_text
        );
    }

    let json: serde_json::Value =
        serde_json::from_str(&response_text).context("Failed to parse rankings JSON")?;

    if let Some(errors) = json.get("errors") {
        anyhow::bail!("GraphQL errors: {}", serde_json::to_string_pretty(errors)?);
    }

    // characterRankings is returned as a JSON scalar — may be a string or an object
    let rankings_json = json
        .pointer("/data/worldData/encounter/characterRankings")
        .context("No characterRankings field in response")?;

    let rankings_value: serde_json::Value = if rankings_json.is_string() {
        let s = rankings_json
            .as_str()
            .context("characterRankings was string but unreadable")?;
        serde_json::from_str(s).context("Failed to parse characterRankings JSON string")?
    } else {
        rankings_json.clone()
    };

    let rankings = match rankings_value.get("rankings").and_then(|v| v.as_array()) {
        Some(r) => r,
        None => {
            let debug_str = serde_json::to_string(&rankings_value).unwrap_or_default();
            anyhow::bail!("No rankings array in WCL response: {}", debug_str);
        }
    };

    if rankings.is_empty() {
        tracing::info!("WCL returned empty rankings for this query.");
        return Ok(());
    }

    tracing::info!("Found {} rankings, fetching talent strings...", rankings.len());

    let mut rank_number = 1usize;

    for rank in rankings.iter() {
        if rank_number > 10 {
            break;
        }

        let name = rank
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        if name == "Anonymous" {
            tracing::debug!("Skipping Anonymous log");
            continue;
        }

        let report_code = rank
            .pointer("/report/code")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let fight_id = rank
            .pointer("/report/fightID")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let log_url = format!(
            "https://www.warcraftlogs.com/reports/{}#fight={}",
            report_code, fight_id
        );

        let talent_string = if !report_code.is_empty() && fight_id > 0 {
            match fetch_talent_string(&client, &token, report_code, fight_id, name).await {
                Ok(s) if !s.is_empty() => s,
                Ok(_) => {
                    tracing::warn!("Empty talent string for rank {} {}", rank_number, name);
                    "[Talent data unavailable]".to_string()
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to fetch talents for rank {} {}: {:#}",
                        rank_number, name, e
                    );
                    "[Talent data unavailable]".to_string()
                }
            }
        } else {
            tracing::warn!("Rank {} {} missing report/fight data", rank_number, name);
            "[Missing report data]".to_string()
        };

        tracing::info!(
            "Rank {} {}: {}",
            rank_number,
            name,
            &talent_string[..talent_string.len().min(40)]
        );

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
