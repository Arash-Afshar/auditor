use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use service::{get_review_state, update_review_state, UpdateReviewState};
use std::net::SocketAddr;

#[derive(Serialize)]
pub struct ReviewState {
    reviewed: Vec<usize>,
    modified: Vec<usize>,
}

#[derive(Deserialize)]
pub struct GetReviewRequest {
    file_name: String,
}

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

async fn handle_get_review_state(
    Json(payload): Json<GetReviewRequest>,
) -> (StatusCode, Json<ReviewState>) {
    match get_review_state(payload.file_name) {
        Ok(state) => (
            StatusCode::CREATED,
            Json(ReviewState {
                reviewed: state.reviewed.into_iter().collect(),
                modified: state.modified.into_iter().collect(),
            }),
        ),
        Err(err) => {
            println!("{}", err.reason);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ReviewState {
                    modified: vec![],
                    reviewed: vec![],
                }),
            )
        }
    }
}

async fn handle_update_review_state(Json(payload): Json<UpdateReviewState>) -> StatusCode {
    match update_review_state(payload) {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            println!("{}", err.reason);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
