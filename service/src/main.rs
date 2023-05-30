use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use service::{get_review_state, update_review_state, ReviewState, UpdateReviewState};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/review", get(handle_get_review_state))
        .route("/review", post(handle_update_review_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Send GET & POST requests to /review"
}

async fn handle_get_review_state(Json(file_name): Json<String>) -> Json<ReviewState> {
    let state = get_review_state(file_name);
    Json(state)
}

async fn handle_update_review_state(Json(payload): Json<UpdateReviewState>) -> StatusCode {
    println!("{:?}", payload);
    match update_review_state(payload) {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            println!("{}", err.reason);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
