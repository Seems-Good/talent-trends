use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

const OAUTH_TOKEN_URL: &str = "https://www.warcraftlogs.com/oauth/token";
const GRAPHQL_ENDPOINT: &str = "https://www.warcraftlogs.com/api/v2/client";

#[derive(Debug, Serialize, Deserialize)]
pub struct TalentData {
    pub name: String,
    pub talent_string: String,
    pub log_url: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
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
    // Query to get actors and talent code
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
    
    // Find the actor ID by name
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
    
    // Now query for the talent import code using the actor ID
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

pub async fn fetch_top_talents(class: &str, spec: &str, encounter_id: i32) -> Result<Vec<TalentData>> {
    let token = get_access_token().await?;
    let client = Client::new();
    
    // Convert class name: "Death_Knight" -> "DeathKnight"
    let class_name = class.replace('_', "");
    
    tracing::info!("Querying: class={}, spec={}, encounter={}", class_name, spec, encounter_id);
    
    // Query for top rankings
    let query = r#"
    query Rankings($encounterId: Int!, $className: String!, $specName: String!) {
      worldData {
        encounter(id: $encounterId) {
          name
          characterRankings(
            className: $className
            specName: $specName
            metric: dps
            difficulty: 5
            page: 1
          )
        }
      }
    }
    "#;
    
    let request = GraphQLRequest {
        query: query.to_string(),
        variables: Some(serde_json::json!({
            "encounterId": encounter_id,
            "className": class_name,
            "specName": spec,
        })),
    };
    
    let response = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(&token)
        .json(&request)
        .send()
        .await
        .context("Failed to send GraphQL request")?;
    
    let status = response.status();
    let response_text = response.text().await?;
    
    if !status.is_success() {
        anyhow::bail!("GraphQL request failed with status {}: {}", status, response_text);
    }
    
    let json: serde_json::Value = serde_json::from_str(&response_text)
        .context("Failed to parse GraphQL response")?;
    
    // Check for GraphQL errors
    if let Some(errors) = json.get("errors") {
        anyhow::bail!("GraphQL errors: {}", serde_json::to_string_pretty(errors)?);
    }
    
    // Parse rankings
    let rankings = json
        .pointer("/data/worldData/encounter/characterRankings/rankings")
        .and_then(|v| v.as_array())
        .context("No rankings found in response")?;
    
    tracing::info!("Found {} rankings, fetching talent strings...", rankings.len());
    
    let mut results = Vec::new();
    
    for (i, rank) in rankings.iter().take(10).enumerate() {
        let name = rank
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        
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
        
        // Fetch talent string
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
        
        tracing::info!("✓ Rank {}: {} - {} chars", i + 1, name, talent_string.len());
        
        results.push(TalentData {
            name: name.to_string(),
            talent_string,
            log_url,
        });
    }
    
    Ok(results)
}




// use anyhow::{Result, Context};
// use serde::{Deserialize, Serialize};
// use reqwest::Client;
// use std::sync::Arc;
// use tokio::sync::RwLock;
//
// const OAUTH_TOKEN_URL: &str = "https://www.warcraftlogs.com/oauth/token";
// const GRAPHQL_ENDPOINT: &str = "https://www.warcraftlogs.com/api/v2/client";
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct TalentData {
//     pub name: String,
//     pub talent_string: String,
//     pub log_url: String,
// }
//
// #[derive(Debug, Deserialize)]
// struct TokenResponse {
//     access_token: String,
//     expires_in: u64,
// }
//
// lazy_static::lazy_static! {
//     static ref TOKEN_CACHE: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
// }
//
// async fn get_access_token() -> Result<String> {
//     {
//         let cache = TOKEN_CACHE.read().await;
//         if let Some(token) = cache.as_ref() {
//             return Ok(token.clone());
//         }
//     }
//
//     let client_id = std::env::var("WCL_CLIENT_ID")
//         .context("WCL_CLIENT_ID not set in .env")?;
//     let client_secret = std::env::var("WCL_CLIENT_SECRET")
//         .context("WCL_CLIENT_SECRET not set in .env")?;
//
//     tracing::info!("Fetching new OAuth token...");
//
//     let client = Client::new();
//     let params = [("grant_type", "client_credentials")];
//
//     let response = client
//         .post(OAUTH_TOKEN_URL)
//         .basic_auth(client_id, Some(client_secret))
//         .form(&params)
//         .send()
//         .await
//         .context("Failed to request OAuth token")?;
//
//     let status = response.status();
//     if !status.is_success() {
//         let error_text = response.text().await.unwrap_or_default();
//         anyhow::bail!("OAuth failed with status {}: {}", status, error_text);
//     }
//
//     let token_resp: TokenResponse = response.json().await
//         .context("Failed to parse OAuth token response")?;
//
//     tracing::info!("✓ OAuth token acquired");
//
//     {
//         let mut cache = TOKEN_CACHE.write().await;
//         *cache = Some(token_resp.access_token.clone());
//     }
//
//     Ok(token_resp.access_token)
// }
//
// #[derive(Serialize)]
// struct GraphQLRequest {
//     query: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     variables: Option<serde_json::Value>,
// }
//
// async fn fetch_talent_string(
//     client: &Client,
//     token: &str,
//     report_code: &str,
//     fight_id: i64,
//     actor_id: i64,
// ) -> Result<String> {
//     // Query to get talent string from a specific fight
//     let query = r#"
//     query TalentString($reportCode: String!, $fightID: Int!, $actorID: Int!) {
//       reportData {
//         report(code: $reportCode) {
//           fights(fightIDs: [$fightID]) {
//             talents(actorID: $actorID)
//           }
//         }
//       }
//     }
//     "#;
//
//     let request = GraphQLRequest {
//         query: query.to_string(),
//         variables: Some(serde_json::json!({
//             "reportCode": report_code,
//             "fightID": fight_id,
//             "actorID": actor_id,
//         })),
//     };
//
//     let response = client
//         .post(GRAPHQL_ENDPOINT)
//         .bearer_auth(token)
//         .json(&request)
//         .send()
//         .await?;
//
//     let json: serde_json::Value = response.json().await?;
//
//     // Extract talent string
//     let talent_string = json
//         .pointer("/data/reportData/report/fights/0/talents")
//         .and_then(|v| v.as_str())
//         .unwrap_or("N/A")
//         .to_string();
//
//     Ok(talent_string)
// }
//
// pub async fn fetch_top_talents(class: &str, spec: &str, encounter_id: i32) -> Result<Vec<TalentData>> {
//     let token = get_access_token().await?;
//     let client = Client::new();
//
//     // Convert class name: "Death_Knight" -> "DeathKnight"
//     let class_name = class.replace('_', "");
//
//     tracing::info!("Querying: class={}, spec={}, encounter={}", class_name, spec, encounter_id);
//
//     // Query for top rankings with actor IDs
//     let query = r#"
//     query Rankings($encounterId: Int!, $className: String!, $specName: String!) {
//       worldData {
//         encounter(id: $encounterId) {
//           name
//           characterRankings(
//             className: $className
//             specName: $specName
//             metric: dps
//             difficulty: 5
//             page: 1
//           )
//         }
//       }
//     }
//     "#;
//
//     let request = GraphQLRequest {
//         query: query.to_string(),
//         variables: Some(serde_json::json!({
//             "encounterId": encounter_id,
//             "className": class_name,
//             "specName": spec,
//         })),
//     };
//
//     let response = client
//         .post(GRAPHQL_ENDPOINT)
//         .bearer_auth(&token)
//         .json(&request)
//         .send()
//         .await
//         .context("Failed to send GraphQL request")?;
//
//     let status = response.status();
//     let response_text = response.text().await?;
//
//     if !status.is_success() {
//         anyhow::bail!("GraphQL request failed with status {}: {}", status, response_text);
//     }
//
//     let json: serde_json::Value = serde_json::from_str(&response_text)
//         .context("Failed to parse GraphQL response")?;
//
//     // Check for GraphQL errors
//     if let Some(errors) = json.get("errors") {
//         anyhow::bail!("GraphQL errors: {}", serde_json::to_string_pretty(errors)?);
//     }
//
//     // Parse rankings
//     let rankings = json
//         .pointer("/data/worldData/encounter/characterRankings/rankings")
//         .and_then(|v| v.as_array())
//         .context("No rankings found in response")?;
//
//     tracing::info!("Found {} rankings, fetching talent strings...", rankings.len());
//
//     let mut results = Vec::new();
//
//     for (i, rank) in rankings.iter().take(10).enumerate() {
//         let name = rank
//             .get("name")
//             .and_then(|v| v.as_str())
//             .unwrap_or("Unknown");
//
//         let report_code = rank
//             .pointer("/report/code")
//             .and_then(|v| v.as_str())
//             .unwrap_or("");
//
//         let fight_id = rank
//             .pointer("/report/fightID")
//             .and_then(|v| v.as_i64())
//             .unwrap_or(0);
//
//         // Get the actor ID from the rankings - this is needed for talent lookup
//         let actor_id = rank
//             .get("gameID")
//             .or_else(|| rank.get("id"))
//             .and_then(|v| v.as_i64())
//             .unwrap_or(0);
//
//         let log_url = format!(
//             "https://www.warcraftlogs.com/reports/{}#fight={}",
//             report_code, fight_id
//         );
//
//         // Fetch talent string for this specific player
//         let talent_string = if !report_code.is_empty() && fight_id > 0 && actor_id > 0 {
//             match fetch_talent_string(&client, &token, report_code, fight_id, actor_id).await {
//                 Ok(s) if !s.is_empty() && s != "N/A" => s,
//                 _ => {
//                     tracing::warn!("Failed to fetch talents for {} (actor: {})", name, actor_id);
//                     "[Talent data unavailable]".to_string()
//                 }
//             }
//         } else {
//             "[Missing report data]".to_string()
//         };
//
//         tracing::debug!("Rank {}: {} - {} chars talent string", i + 1, name, talent_string.len());
//
//         results.push(TalentData {
//             name: name.to_string(),
//             talent_string,
//             log_url,
//         });
//     }
//
//     Ok(results)
// }
