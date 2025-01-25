use aws_config::BehaviorVersion;
use aws_sdk_dsql::auth_token::{AuthTokenGenerator, Config};
use axum::{
     http::{StatusCode}, response::{ Json}, routing::{get,post}, Router
};
use serde_json::Value;
use std::{ net::SocketAddr};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use opentelemetry::{global, KeyValue, trace::Tracer};
use opentelemetry_sdk::{trace::{self, RandomIdGenerator, Sampler}, Resource};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use std::time::Duration;


#[tokio::main]
async fn main() {


    // トレーシングログの出力設定を行っている。
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_line_number(true)
                .with_file(true)
        )
        .init();


    let app = app().await;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}



async fn app() -> Router   {


    Router::new()
        .route("/", get(handler))
        .route("/puthistory", post(put_rss))
        .route("/gethistory", post(get_rss))

}

async fn handler() -> &'static str {
    "Hello, World!"
}

async fn put_rss(body_json: Json<RSSHistory>) -> (StatusCode) {

    let cfg = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let generator: AuthTokenGenerator = AuthTokenGenerator::new(
        Config::builder()
            .hostname("qaabtwo5ilyhegvfnfbflqafdy.dsql.us-east-2.on.aws")
            .build()
            .expect("cfg is valid"),
    );
    let token: aws_sdk_dsql::auth_token::AuthToken = generator.db_connect_admin_auth_token(&cfg).await.unwrap();
    println!("{token}");

    // Setup connections
    let connection_options = PgConnectOptions::new()
        .host("qaabtwo5ilyhegvfnfbflqafdy.dsql.us-east-2.on.aws")
        .port(5432)
        .database("postgres")
        .username("admin")
        .password(token.as_str())
        .ssl_mode(sqlx::postgres::PgSslMode::VerifyFull);

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(connection_options.clone())
        .await
        .unwrap();

    let rsshistory = sqlx::query(
        "INSERT INTO whatnewrsshistory (url, notifier_name, category, title) VALUES ($1, $2, $3, $4) RETURNING *"
    )
        .bind(body_json.url.clone())
        .bind(body_json.notifier_name.clone())
        .bind(body_json.category.clone())
        .bind(body_json.title.clone())
        .execute(&pool)
        .await;


    pool.close().await;

    return (
        StatusCode::OK
    )

}



async fn get_rss(body_json: Json<Value>) -> (StatusCode, Json<RSSHistory>) {

    let cfg = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let generator = AuthTokenGenerator::new(
        Config::builder()
            .hostname("qaabtwo5ilyhegvfnfbflqafdy.dsql.us-east-2.on.aws")
            .build()
            .expect("cfg is valid"),
    );
    let token: aws_sdk_dsql::auth_token::AuthToken = generator.db_connect_admin_auth_token(&cfg).await.unwrap();
    println!("{token}");

    // Setup connections
    let connection_options = PgConnectOptions::new()
        .host("qaabtwo5ilyhegvfnfbflqafdy.dsql.us-east-2.on.aws")
        .port(5432)
        .database("postgres")
        .username("admin")
        .password(token.as_str())
        .ssl_mode(sqlx::postgres::PgSslMode::VerifyFull);

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(connection_options.clone())
        .await
        .unwrap();

    let rsshistory = sqlx::query_as::<_, RSSHistory>("SELECT * FROM whatnewrsshistory")
        .fetch_one(&pool)
        .await
        .unwrap();



    pool.close().await;

    return (
        StatusCode::OK,
        Json(rsshistory)
    )

}


#[derive(serde::Serialize)]
struct Body {
    Body: String
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow)]
struct RSSHistory {
    url: String,
    notifier_name: String,
    category: String,
    //pubtime: chrono::DateTime<chrono::Utc>,
    title: String
}
