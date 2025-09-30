use axum::{
    extract::Query,
    response::{Html, sse::{Event, Sse}, Response},
    routing::get,
    Router,
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio_stream::StreamExt as _;

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
                .with_target(false)
                .compact()
        )
        .init();

    let config = ClassSpecs::load();
    tracing::info!("Loaded {} classes", config.classes.len());

    let app = Router::new()
        .route("/", get(home))
        .route("/api/talents", get(get_talents_sse));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("🚀 Server listening on http://{}", addr);
    
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



// use axum::{
//     extract::Query,
//     response::Html,
//     routing::get,
//     Router,
// };
// use serde::{Deserialize, Serialize};
// use std::net::SocketAddr;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
//
// mod config;
// mod warcraftlogs;
// mod templates;
//
// use config::ClassSpecs;
//
// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     // Load .env file
//     dotenvy::dotenv().ok();
//
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::fmt::layer()
//                 .with_target(false)
//                 .compact()
//         )
//         .init();
//
//     let config = ClassSpecs::load();
//     tracing::info!("Loaded {} classes", config.classes.len());
//
//     let app = Router::new()
//         .route("/", get(home))
//         .route("/api/talents", get(get_talents));
//
//     let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
//     tracing::info!("Server listening on http://{}", addr);
//
//     let listener = tokio::net::TcpListener::bind(addr).await?;
//     axum::serve(listener, app).await?;
//
//     Ok(())
// }
//
// async fn home() -> Html<String> {
//     let config = ClassSpecs::load();
//     Html(templates::home(&config))
// }
//
// #[derive(Deserialize)]
// struct TalentQuery {
//     class: String,
//     spec: String,
//     encounter: i32,
//     region: String,
// }
//
// async fn get_talents(Query(params): Query<TalentQuery>) -> Html<String> {
//     let region_display = if params.region == "all" {
//         "All Regions"
//     } else {
//         &params.region
//     };
//
//     tracing::info!(
//         "Fetching talents for {} {} on encounter {} in {} region.",
//         params.class,
//         params.spec,
//         params.encounter,
//         region_display
//     );
//
//     let region = if params.region == "all" {
//         None
//     } else {
//         Some(params.region.as_str())
//     };
//
//
//     match warcraftlogs::fetch_top_talents(&params.class, &params.spec, params.encounter, region).await {
//         Ok(data) => Html(templates::render_talents(&data)),
//         Err(e) => {
//             tracing::error!("Failed to fetch talents: {:#}", e);
//             Html(format!(r#"<div class="error">Error: {}</div>"#, e))
//         }
//     }
// }
//

