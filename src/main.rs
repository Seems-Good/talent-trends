use axum::{
    extract::Query,
    response::{
        Html,
        sse::{Event, Sse},
    },
    routing::get,
    Router,
};
use futures::stream::Stream;
use serde::Deserialize;
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod style;
mod templates;
mod warcraftlogs;

use config::{ClassSpecs, Settings};
use warcraftlogs::TalentDataWithRank;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .compact(),
        )
        .init();

    let config = ClassSpecs::load();

    for (class_name, class_data) in &config.classes {
        let color_len = class_data.color.len();
        let pretty_len = class_data.pretty_color.len();

        tracing::trace!(
            class = %class_name,
            color_len = color_len,
            pretty_len = pretty_len,
            "Checking color lengths per class"
        );
        assert_eq!(
            color_len, pretty_len,
            "Mismatch in lengths for class '{}' (color: {}, pretty_color: {})",
            class_name, color_len, pretty_len
        );
    }

    tracing::info!(
        "Loaded {} classes from `classes.toml` config.",
        config.classes.len(),
    );

    let app = Router::new()
        .route("/", get(home))
        .route("/api/talents", get(get_talents_sse));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Deserialize)]
struct TalentQuery {
    class: String,
    spec: String,
    encounter: i32,
    region: String,
    mode: String,
}

async fn home() -> Html<String> {
    let config = ClassSpecs::load();
    Html(templates::home(&config))
}

async fn get_talents_sse(
    Query(params): Query<TalentQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let region_display = if params.region == "all" {
        "All Regions".to_string()
    } else {
        params.region.clone()
    };

    let settings = Settings::load();

    let difficulty = ClassSpecs::get_modes()
        .into_iter()
        .find(|m| m.name == params.mode)
        .map(|m| m.difficulty)
        .unwrap_or_else(|| settings.default_difficulty());

    let partition = settings.current_partition();

    tracing::info!(
        "Fetching talents for {} {} on encounter {} (region: {}, mode: {}, difficulty: {}, partition: {:?})",
        params.class,
        params.spec,
        params.encounter,
        region_display,
        params.mode,
        difficulty,
        partition,
    );

    let region = if params.region == "all" {
        None
    } else {
        Some(params.region.clone())
    };

    let stream = async_stream::stream! {
        match warcraftlogs::fetch_top_talents_stream(
            &params.class,
            &params.spec,
            params.encounter,
            region.as_deref(),
            difficulty,
            partition,
        ).await {
            Ok(mut receiver) => {
                while let Some(result) = receiver.recv().await {
                    let result: Result<TalentDataWithRank, _> = result;
                    match result {
                        Ok(talent_data) => {
                            let html = templates::render_talent_entry(&talent_data);
                            yield Ok(Event::default().data(html));
                        }
                        Err(e) => {
                            tracing::error!("Worker error while streaming talents: {:#}", e);
                            let error_html = format!(r#"<div class="error">Error: {}</div>"#, e);
                            yield Ok(Event::default().data(error_html));
                            break;
                        }
                    }
                }
                yield Ok(Event::default().event("complete").data("done"));
            }
            Err(e) => {
                tracing::error!("Failed to start fetch_top_talents_stream: {:#}", e);
                let error_html = format!(r#"<div class="error">Error: {}</div>"#, e);
                yield Ok(Event::default().data(error_html));
                yield Ok(Event::default().event("complete").data("done"));
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive"),
    )
}
