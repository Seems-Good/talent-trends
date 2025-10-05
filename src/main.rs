use axum::{
    extract::Query,
    response::{Html, sse::{Event, Sse}},
    routing::get,
    Router,
};
use futures::stream::Stream;
use serde::Deserialize;
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod warcraftlogs;
mod templates;

use config::ClassSpecs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .compact()
        )
        .init();

    let config = ClassSpecs::load();

    // Itterate over classes and verify we loaded correct data from `classes.toml`
    for (class_name, class_data) in &config.classes {
        let color_len = class_data.color.len();
        let pretty_len = class_data.pretty_color.len();

        // Trace each class color lengths for verbose debugging
        tracing::trace!(
            class = %class_name,
            color_len = color_len,
            pretty_len = pretty_len,
            "Checking color lengths per class"
        );
        // Make sure we have a pretty-color for each color code.
        assert_eq!(
            color_len, pretty_len,
            "Mismatch in lengths for class '{}' (color: {}, pretty_color: {})",
            class_name, color_len, pretty_len
        );
    }

    tracing::info!("Loaded {} classes from `classes.toml` config.", 
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
}

async fn home() -> Html<String> {
    let config = ClassSpecs::load();
    Html(templates::home(&config))
}

async fn get_talents_sse(
    Query(params): Query<TalentQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let region_display = if params.region == "all" { 
        "All Regions" 
    } else { 
        &params.region 
    };
    
    tracing::info!(
        "Fetching talents for {} {} on encounter {} (region: {})",
        params.class,
        params.spec,
        params.encounter,
        region_display
    );
    
    let region = if params.region == "all" {
        None
    } else {
        Some(params.region.clone())
    };
    
    let stream = async_stream::stream! {
        match warcraftlogs::fetch_top_talents_stream(&params.class, &params.spec, params.encounter, region.as_deref()).await {
            Ok(mut receiver) => {
                while let Some(result) = receiver.recv().await {
                    match result {
                        Ok(talent_data) => {
                            let html = templates::render_talent_entry(&talent_data);
                            yield Ok(Event::default().data(html));
                        }
                        Err(e) => {
                            let error_html = format!(r#"<div class="error">Error: {}</div>"#, e);
                            yield Ok(Event::default().data(error_html));
                        }
                    }
                }
                
                // Send completion event
                yield Ok(Event::default().event("complete").data("done"));
            }
            Err(e) => {
                tracing::error!("Failed to fetch talents: {:#}", e);
                let error_html = format!(r#"<div class="error">Error: {}</div>"#, e);
                yield Ok(Event::default().data(error_html));
                yield Ok(Event::default().event("complete").data("done"));
            }
        }
    };
    
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive")
    )
}
