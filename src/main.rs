use axum::{
    extract::Query,
    response::Html,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod warcraftlogs;
mod templates;

use config::ClassSpecs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
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
        .route("/api/talents", get(get_talents));

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
}

async fn home() -> Html<String> {
    let config = ClassSpecs::load();
    Html(templates::home(&config))
}

async fn get_talents(Query(params): Query<TalentQuery>) -> Html<String> {
    tracing::info!("Fetching talents for {} {}", params.class, params.spec);
    
    match warcraftlogs::fetch_top_talents(&params.class, &params.spec).await {
        Ok(data) => Html(templates::render_talents(&data)),
        Err(e) => {
            tracing::error!("Failed to fetch talents: {}", e);
            Html(format!(r#"<p style="color: #e06c75;">Error: {}</p>"#, e))
        }
    }
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
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::fmt::layer()
//                 .with_target(false)
//                 .compact()
//         )
//         .init();
//
//     // Load config once at startup
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
// #[derive(Deserialize)]
// struct TalentQuery {
//     class: String,
//     spec: String,
// }
//
// async fn home() -> Html<String> {
//     let config = ClassSpecs::load();
//     Html(templates::home(&config))
// }
//
// async fn get_talents(Query(params): Query<TalentQuery>) -> Html<String> {
//     tracing::info!("Fetching talents for {} {}", params.class, params.spec);
//
//     match warcraftlogs::fetch_top_talents(&params.class, &params.spec).await {
//         Ok(data) => Html(templates::render_talents(&data)),
//         Err(e) => {
//             tracing::error!("Failed to fetch talents: {}", e);
//             Html(format!(r#"<p style="color: #e06c75;">Error: {}</p>"#, e))
//         }
//     }
// }
//
//
//

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
// mod warcraftlogs;
// mod templates;
//
// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     tracing_subscriber::registry()
//         .with(tracing_subscriber::fmt::layer())
//         .init();
//
//     let app = Router::new()
//         .route("/", get(home))
//         .route("/api/talents", get(get_talents));
//
//     let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
//     tracing::info!("listening on {}", addr);
//
//     let listener = tokio::net::TcpListener::bind(addr).await?;
//     axum::serve(listener, app).await?;
//
//     Ok(())
// }
//
// #[derive(Deserialize)]
// struct TalentQuery {
//     class: String,
//     spec: String,
// }
//
// async fn home() -> Html<&'static str> {
//     Html(templates::HOME)
// }
//
// async fn get_talents(Query(params): Query<TalentQuery>) -> Html<String> {
//     match warcraftlogs::fetch_top_talents(&params.class, &params.spec).await {
//         Ok(data) => Html(templates::render_talents(&data)),
//         Err(e) => {
//             tracing::error!("Failed to fetch talents: {}", e);
//             Html(format!("<p>Error: {}</p>", e))
//         }
//     }
// }

