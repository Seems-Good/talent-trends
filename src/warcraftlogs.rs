use anyhow::Result;
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

// Token cache (you'll want to refresh before expiry)
lazy_static::lazy_static! {
    static ref TOKEN_CACHE: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
}

async fn get_access_token() -> Result<String> {
    // Check cache first
    {
        let cache = TOKEN_CACHE.read().await;
        if let Some(token) = cache.as_ref() {
            return Ok(token.clone());
        }
    }
    
    let client_id = std::env::var("WCL_CLIENT_ID")?;
    let client_secret = std::env::var("WCL_CLIENT_SECRET")?;
    
    let client = Client::new();
    let params = [
        ("grant_type", "client_credentials"),
    ];
    
    let response = client
        .post(OAUTH_TOKEN_URL)
        .basic_auth(client_id, Some(client_secret))
        .form(&params)
        .send()
        .await?;
    
    let token_resp: TokenResponse = response.json().await?;
    
    // Cache the token
    {
        let mut cache = TOKEN_CACHE.write().await;
        *cache = Some(token_resp.access_token.clone());
    }
    
    Ok(token_resp.access_token)
}

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: serde_json::Value,
}

pub async fn fetch_top_talents(class: &str, spec: &str) -> Result<Vec<TalentData>> {
    let token = get_access_token().await?;
    let client = Client::new();
    
    // GraphQL query for top rankings with talent data
    let query = r#"
    query($className: String!, $specName: String!) {
        worldData {
            encounter(id: 2917) {
                characterRankings(
                    className: $className
                    specName: $specName
                    metric: dps
                    page: 1
                    pageSize: 10
                ) {
                    rankings {
                        character {
                            name
                        }
                        report {
                            code
                        }
                        startTime
                    }
                }
            }
        }
    }
    "#;
    
    let request = GraphQLRequest {
        query: query.to_string(),
        variables: serde_json::json!({
            "className": class,
            "specName": spec,
        }),
    };
    
    let response = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(token)
        .json(&request)
        .send()
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    
    // TODO: Parse response and extract actual talent strings
    // (Requires digging into combatantInfo in the report data)
    
    Ok(vec![])
}

