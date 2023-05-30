use axum::{
    extract::Query,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use service::{db::DB, get_review_state, git::Git, update_review_state, UpdateReviewState};
use std::{collections::HashMap, net::SocketAddr};

#[derive(Serialize)]
pub struct ReviewState {
    reviewed: Vec<usize>,
    modified: Vec<usize>,
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
    Query(query): Query<HashMap<String, String>>,
) -> (StatusCode, Json<ReviewState>) {
    // TODO: find a way to share these across requests and handle the unwraps
    println!("GET Request received: {:?}", query);
    let git = Git::new("../".to_string()).unwrap();
    let mut db = DB::new("main.db".to_string()).unwrap();
    let file_name = query.get(&"file_name".to_string());
    if file_name.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ReviewState {
                reviewed: vec![],
                modified: vec![],
            }),
        );
    }
    let file_name = file_name.unwrap();
    match get_review_state(file_name, &mut db, &git) {
        Ok(state) => {
            db.save("main.db".to_string()).unwrap();
            (
                StatusCode::CREATED,
                Json(ReviewState {
                    reviewed: state.reviewed.into_iter().collect(),
                    modified: state.modified.into_iter().collect(),
                }),
            )
        }
        Err(err) => {
            println!("{}", err.message);
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
    // TODO: find a way to share these across requests and handle the unwraps
    println!("POST Request received: {:?}", payload);
    let git = Git::new("../".to_string()).unwrap();
    let mut db = DB::new("main.db".to_string()).unwrap();
    match update_review_state(payload, &mut db, &git) {
        Ok(_) => {
            db.save("main.db".to_string()).unwrap();
            StatusCode::CREATED
        }
        Err(err) => {
            println!("{}", err.message);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
