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

pub async fn fetch_top_talents(class: &str, spec: &str, encounter_id: i32) -> Result<Vec<TalentData>> {
    let token = get_access_token().await?;
    let client = Client::new();
    
    // Query for top 10 rankings with report codes
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
            "className": class,
            "specName": spec,
        })),
    };
    
    tracing::debug!("GraphQL request: {:?}", serde_json::to_string_pretty(&request)?);
    
    let response = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(&token)
        .json(&request)
        .send()
        .await
        .context("Failed to send GraphQL request")?;
    
    let status = response.status();
    let response_text = response.text().await?;
    
    tracing::debug!("GraphQL response ({}): {}", status, response_text);
    
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
    
    tracing::info!("Found {} rankings", rankings.len());
    
    let mut results = Vec::new();
    
    for (i, rank) in rankings.iter().take(10).enumerate() {
        let name = rank
            .pointer("/character/name")
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
        
        // TODO: Fetch actual talent string from combatantInfo
        let talent_string = format!("[Placeholder talent data for rank {}]", i + 1);
        
        results.push(TalentData {
            name: name.to_string(),
            talent_string,
            log_url,
        });
    }
    
    Ok(results)
}

// Test function to verify API connectivity
pub async fn test_connection() -> Result<()> {
    tracing::info!("Testing WarcraftLogs API connection...");
    
    let token = get_access_token().await?;
    let client = Client::new();
    
    // Simple test query
    let query = r#"
    {
      worldData {
        expansion(id: 10) {
          name
          zones {
            id
            name
          }
        }
      }
    }
    "#;
    
    let request = GraphQLRequest {
        query: query.to_string(),
        variables: None,
    };
    
    let response = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(&token)
        .json(&request)
        .send()
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    tracing::info!("API test response: {}", serde_json::to_string_pretty(&json)?);
    
    Ok(())
}



// use anyhow::Result;
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
// // Token cache (you'll want to refresh before expiry)
// lazy_static::lazy_static! {
//     static ref TOKEN_CACHE: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
// }
//
// async fn get_access_token() -> Result<String> {
//     // Check cache first
//     {
//         let cache = TOKEN_CACHE.read().await;
//         if let Some(token) = cache.as_ref() {
//             return Ok(token.clone());
//         }
//     }
//
//     let client_id = std::env::var("WCL_CLIENT_ID")?;
//     let client_secret = std::env::var("WCL_CLIENT_SECRET")?;
//
//     let client = Client::new();
//     let params = [
//         ("grant_type", "client_credentials"),
//     ];
//
//     let response = client
//         .post(OAUTH_TOKEN_URL)
//         .basic_auth(client_id, Some(client_secret))
//         .form(&params)
//         .send()
//         .await?;
//
//     let token_resp: TokenResponse = response.json().await?;
//
//     // Cache the token
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
//     variables: serde_json::Value,
// }
//
// pub async fn fetch_top_talents(class: &str, spec: &str) -> Result<Vec<TalentData>> {
//     let token = get_access_token().await?;
//     let client = Client::new();
//
//     // GraphQL query for top rankings with talent data
//     let query = r#"
//     query($className: String!, $specName: String!) {
//         worldData {
//             encounter(id: 3129) {
//                 characterRankings(
//                     className: $className
//                     specName: $specName
//                     metric: dps
//                     page: 1
//                     pageSize: 10
//                 ) {
//                     rankings {
//                         character {
//                             name
//                         }
//                         report {
//                             code
//                         }
//                         startTime
//                     }
//                 }
//             }
//         }
//     }
//     "#;
//
//     let request = GraphQLRequest {
//         query: query.to_string(),
//         variables: serde_json::json!({
//             "className": class,
//             "specName": spec,
//         }),
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
//     // TODO: Parse response and extract actual talent strings
//     // (Requires digging into combatantInfo in the report data)
//
//     Ok(vec![])
// }

