use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use service::{db::DB, get_review_state, git::Git, update_review_state, UpdateReviewState};
use std::{collections::HashMap, env, net::SocketAddr};

#[derive(Clone)]
pub struct AppState {
    repo_path: String,
    db_path: String,
}

#[derive(Serialize)]
pub struct ReviewState {
    reviewed: Vec<usize>,
    modified: Vec<usize>,
}

const REPO_PATH_ENV: &str = "REPO_PATH";
const DB_PATH_ENV: &str = "DB_PATH";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app_state = match (env::var(REPO_PATH_ENV), env::var(DB_PATH_ENV)) {
        // TODO: get dir exclusion as well
        (Ok(repo_path), Ok(db_path)) => AppState { repo_path, db_path },
        _ => panic!(
            "Environment paths not set {}, {}",
            REPO_PATH_ENV, DB_PATH_ENV
        ),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/review", get(handle_get_review_state))
        .route("/review", post(handle_update_review_state))
        .with_state(app_state);

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
    State(state): State<AppState>,
) -> (StatusCode, Json<ReviewState>) {
    println!("GET Request received: {:?}", query);
    let git = Git::new(&state.repo_path).unwrap();
    let mut db = DB::new(state.db_path).unwrap();
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
    let file_name = file_name
        .unwrap()
        .replace(&state.repo_path, &"".to_string());
    match get_review_state(&file_name, &mut db, &git) {
        Ok(state) => {
            db.save().unwrap();
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

async fn handle_update_review_state(
    State(state): State<AppState>,
    Json(payload): Json<UpdateReviewState>,
) -> StatusCode {
    println!("POST Request received: {:?}", payload);
    let git = Git::new(&state.repo_path).unwrap();
    let mut db = DB::new(state.db_path).unwrap();
    let mut payload = payload;
    payload.file_name = payload.file_name.replace(&state.repo_path, &"".to_string());
    match update_review_state(payload, &mut db, &git) {
        Ok(_) => {
            db.save().unwrap();
            StatusCode::CREATED
        }
        Err(err) => {
            println!("{}", err.message);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
