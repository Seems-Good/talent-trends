use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

const OAUTH_TOKEN_URL: &str = "https://www.warcraftlogs.com/oauth/token";
const GRAPHQL_ENDPOINT: &str = "https://www.warcraftlogs.com/api/v2/client";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastEvent {
    pub t: i64,
    pub id: u64,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TalentData {
    pub name: String,
    pub talent_string: String,
    pub log_url: String,
    pub fight_duration_ms: i64,
    pub cast_events: Vec<CastEvent>,
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

struct TalentResult {
    talent_string: String,
    fight_duration_ms: i64,
    cast_events: Vec<CastEvent>,
}

async fn fetch_talent_and_events(
    client: &Client,
    token: &str,
    report_code: &str,
    fight_id: i64,
    player_name: &str,
) -> Result<TalentResult> {
    // ── Step 1: resolve actor ID ──────────────────────────────────────────────
    let actor_json: serde_json::Value = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(token)
        .json(&GraphQLRequest {
            query: r#"
            query GetActors($reportCode: String!) {
              reportData {
                report(code: $reportCode) {
                  masterData(translate: true) {
                    actors(type: "Player") { id name }
                  }
                }
              }
            }"#.to_string(),
            variables: Some(serde_json::json!({ "reportCode": report_code })),
        })
        .send().await.context("actor lookup send")?
        .json().await.context("actor lookup parse")?;

    let actors = actor_json
        .pointer("/data/reportData/report/masterData/actors")
        .and_then(|v| v.as_array())
        .context("No actors array in masterData")?;

    let actor_id = actors
        .iter()
        .find(|a| {
            a.get("name").and_then(|n| n.as_str())
                .map(|n| n == player_name).unwrap_or(false)
        })
        .and_then(|a| a.get("id"))
        .and_then(|id| id.as_i64())
        .with_context(|| format!("Actor '{}' not found in masterData", player_name))?;

    tracing::debug!("Resolved actor '{}' -> ID {}", player_name, actor_id);

    // ── Step 2: talent + table (name/icon map) + flat cast events ─────────────
    let combined: serde_json::Value = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(token)
        .json(&GraphQLRequest {
            query: r#"
            query GetAll($code: String!, $ids: [Int]!, $src: Int!) {
              reportData {
                report(code: $code) {
                  fights(fightIDs: $ids) {
                    startTime
                    endTime
                    talentImportCode(actorID: $src)
                  }
                  table(
                    fightIDs: $ids
                    sourceID: $src
                    dataType: Casts
                    translate: true
                  )
                  events(
                    fightIDs: $ids
                    sourceID: $src
                    dataType: Casts
                    limit: 10000
                  ) {
                    data
                    nextPageTimestamp
                  }
                }
              }
            }"#.to_string(),
            variables: Some(serde_json::json!({
                "code": report_code,
                "ids":  [fight_id as i32],
                "src":  actor_id as i32,
            })),
        })
        .send().await.context("combined query send")?
        .json().await.context("combined query parse")?;

    let report = combined
        .pointer("/data/reportData/report")
        .context("No report in combined response")?;

    // ── Fight timing + talent string ──────────────────────────────────────────
    let fight = report.pointer("/fights/0").context("No fight[0]")?;

    let fight_start       = fight.get("startTime").and_then(|v| v.as_i64()).unwrap_or(0);
    let fight_end         = fight.get("endTime").and_then(|v| v.as_i64()).unwrap_or(0);
    let fight_duration_ms = fight_end - fight_start;

    let talent_code = fight
        .get("talentImportCode")
        .and_then(|v| v.as_str())
        .context("No talentImportCode")?;

    // ── Build guid → (name, icon) map from table entries ─────────────────────
    let table_raw = report.get("table").cloned().unwrap_or(serde_json::Value::Null);
    let table_value: serde_json::Value = if table_raw.is_string() {
        serde_json::from_str(table_raw.as_str().unwrap_or("{}")).unwrap_or(serde_json::Value::Null)
    } else {
        table_raw
    };

    let entries = table_value
        .pointer("/data/entries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut ability_map: HashMap<u64, (String, String)> = HashMap::new();

    for entry in &entries {
        if let (Some(guid), Some(name), Some(icon)) = (
            entry.get("guid").and_then(|v| v.as_u64()),
            entry.get("name").and_then(|v| v.as_str()),
            entry.get("abilityIcon").and_then(|v| v.as_str()),
        ) {
            ability_map.entry(guid).or_insert_with(|| (name.to_string(), icon.to_string()));
        }

        if let Some(subs) = entry.get("subentries").and_then(|v| v.as_array()) {
            for sub in subs {
                if let (Some(guid), Some(name), Some(icon)) = (
                    sub.get("guid").and_then(|v| v.as_u64()),
                    sub.get("name").and_then(|v| v.as_str()),
                    sub.get("abilityIcon").and_then(|v| v.as_str()),
                ) {
                    ability_map.entry(guid).or_insert_with(|| (name.to_string(), icon.to_string()));
                }
            }
        }
    }

    tracing::debug!("Built ability map: {} entries for {}", ability_map.len(), player_name);

    // ── Parse flat cast events, join with ability map ─────────────────────────
    let events_raw = report
        .pointer("/events/data")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let events_array: Vec<serde_json::Value> = match events_raw {
        serde_json::Value::Array(arr) => arr,
        serde_json::Value::String(s)  => serde_json::from_str(&s).unwrap_or_default(),
        _                             => vec![],
    };

    let cast_events: Vec<CastEvent> = events_array
        .iter()
        .filter(|ev| ev.get("type").and_then(|v| v.as_str()) == Some("cast"))
        .filter_map(|ev| {
            let timestamp = ev.get("timestamp")?.as_i64()?;
            let id        = ev.get("abilityGameID")?.as_u64()?;
            if id < 100 { return None; }
            let (name, icon) = ability_map.get(&id)?.clone();
            Some(CastEvent { t: timestamp - fight_start, id, name, icon })
        })
        .collect();

    tracing::info!(
        "Parsed {} cast events for {} ({} raw, {} abilities, {}ms)",
        cast_events.len(), player_name, events_array.len(), ability_map.len(), fight_duration_ms
    );

    Ok(TalentResult { talent_string: talent_code.to_string(), fight_duration_ms, cast_events })
}

pub async fn fetch_top_talents_stream(
    class: &str,
    spec: &str,
    encounter_id: i32,
    region: Option<&str>,
    difficulty: i32,
    partition: Option<i32>,
    metric: &str,
) -> Result<mpsc::Receiver<Result<TalentDataWithRank>>> {
    let (tx, rx) = mpsc::channel(10);

    let class   = class.to_string();
    let spec    = spec.to_string();
    let region  = region.map(|s| s.to_string());
    let metric  = metric.to_string();

    tokio::spawn(async move {
        if let Err(e) = fetch_and_stream_talents(
            &tx, &class, &spec, encounter_id, region.as_deref(), difficulty, partition, &metric,
        ).await {
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
    metric: &str,
) -> Result<()> {
    let token  = get_access_token().await?;
    let client = Client::new();

    let class_name     = class.replace('_', "");
    let region_display = region.unwrap_or("all");

    // Validate metric to avoid injecting arbitrary GraphQL
    let safe_metric = match metric {
        "hps" | "tankhps" => metric,
        _                 => "dps",
    };

    tracing::info!(
        "Querying {} {} encounter {} region {} difficulty {} partition {:?} metric {}",
        class_name, spec, encounter_id, region_display, difficulty, partition, safe_metric
    );

    let partition_arg = match partition {
        Some(p) => format!("partition: {}", p),
        None    => String::new(),
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
                metric: {metric}
                difficulty: $difficulty
                page: 1
                {partition_arg}
              )
            }}
          }}
        }}
        "#,
        metric        = safe_metric,
        partition_arg = partition_arg,
    );

    let mut variables = serde_json::json!({
        "encounterId": encounter_id,
        "className":   class_name,
        "specName":    spec,
        "difficulty":  difficulty,
    });
    if let Some(r) = region {
        variables["serverRegion"] = serde_json::Value::String(r.to_string());
    }

    let response = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(&token)
        .json(&GraphQLRequest { query, variables: Some(variables) })
        .send().await.context("rankings send")?;

    let status        = response.status();
    let response_text = response.text().await?;
    if !status.is_success() {
        anyhow::bail!("GraphQL request failed {}: {}", status, response_text);
    }

    let json: serde_json::Value =
        serde_json::from_str(&response_text).context("rankings parse")?;

    if let Some(errors) = json.get("errors") {
        anyhow::bail!("GraphQL errors: {}", serde_json::to_string_pretty(errors)?);
    }

    let rankings_json = json
        .pointer("/data/worldData/encounter/characterRankings")
        .context("No characterRankings field")?;

    let rankings_value: serde_json::Value = if rankings_json.is_string() {
        serde_json::from_str(rankings_json.as_str().context("characterRankings unreadable")?)
            .context("characterRankings parse")?
    } else {
        rankings_json.clone()
    };

    let rankings = match rankings_value.get("rankings").and_then(|v| v.as_array()) {
        Some(r) => r,
        None => anyhow::bail!(
            "No rankings array: {}",
            serde_json::to_string(&rankings_value).unwrap_or_default()
        ),
    };

    if rankings.is_empty() {
        tracing::info!("Empty rankings.");
        return Ok(());
    }

    tracing::info!("Found {} rankings, fetching data...", rankings.len());

    let mut rank_number = 1usize;

    for rank in rankings.iter() {
        if rank_number > 10 { break; }

        let name        = rank.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
        if name == "Anonymous" { continue; }

        let report_code = rank.pointer("/report/code").and_then(|v| v.as_str()).unwrap_or("");
        let fight_id    = rank.pointer("/report/fightID").and_then(|v| v.as_i64()).unwrap_or(0);

        let log_url = format!(
            "https://www.warcraftlogs.com/reports/{}#fight={}",
            report_code, fight_id
        );

        let (talent_string, fight_duration_ms, cast_events) =
            if !report_code.is_empty() && fight_id > 0 {
                match fetch_talent_and_events(&client, &token, report_code, fight_id, name).await {
                    Ok(r) => (r.talent_string, r.fight_duration_ms, r.cast_events),
                    Err(e) => {
                        tracing::warn!("Rank {} {} failed: {:#}", rank_number, name, e);
                        ("[Talent data unavailable]".to_string(), 0, vec![])
                    }
                }
            } else {
                ("[Missing report data]".to_string(), 0, vec![])
            };

        tracing::info!("Rank {} {} — {} cast events", rank_number, name, cast_events.len());

        if tx.send(Ok(TalentDataWithRank {
            rank: rank_number,
            data: TalentData { name: name.to_string(), talent_string, log_url, fight_duration_ms, cast_events },
        })).await.is_err() {
            break;
        }

        rank_number += 1;
    }

    Ok(())
}
