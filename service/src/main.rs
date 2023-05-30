use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use service::{
    db::DB, get_review_state, git::Git, update_review_state, FileComments, UpdateReviewState,
};
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

#[derive(Deserialize, Debug)]
pub struct CreateComment {
    file_name: String,
    line_number: usize,
    body: String,
    author: String,
}

#[derive(Deserialize)]
pub struct UpdateComment {
    line_number: usize,
    comment_id: String,
    body: String,
    author: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteComment {
    file_name: String,
    line_number: usize,
    comment_id: String,
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
        .route("/reviews", post(handle_update_review_state))
        .route("/reviews", get(handle_get_review_state))
        .route("/comments", post(handle_create_comment))
        .route("/comments", get(handle_get_comments))
        .route("/comments", delete(handle_delete_comment))
        //.route("/comments/:comment_id", put(handle_update_comment))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Send requests to /reviews and /comments endpoints"
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

async fn handle_create_comment(
    State(state): State<AppState>,
    Json(payload): Json<CreateComment>,
) -> (StatusCode, Json<String>) {
    println!("POST Request received: {:?}", payload);
    let mut db = DB::new(state.db_path).unwrap();
    let file_name = payload.file_name.replace(&state.repo_path, &"".to_string());
    match db.add_new_comment(file_name, payload.line_number, payload.body, payload.author) {
        Ok(new_comment_id) => {
            db.save().unwrap();
            (StatusCode::CREATED, Json(new_comment_id))
        }
        Err(err) => {
            println!("{:?}", err);
            (StatusCode::BAD_REQUEST, Json("".to_string()))
        }
    }
}

async fn handle_update_comment(
    State(state): State<AppState>,
    Path(file_name): Path<String>,
    Json(payload): Json<UpdateComment>,
) -> StatusCode {
    let mut db = DB::new(state.db_path).unwrap();
    match db.update_comment(
        file_name,
        payload.comment_id,
        payload.line_number,
        payload.body,
        payload.author,
    ) {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            println!("{}", err.message);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn handle_delete_comment(
    State(state): State<AppState>,
    Json(payload): Json<DeleteComment>,
) -> StatusCode {
    println!("DELETE Request received: {:?}", payload);
    let mut db = DB::new(state.db_path).unwrap();
    let file_name = payload.file_name.replace(&state.repo_path, &"".to_string());
    match db.delete_comment(file_name, payload.comment_id, payload.line_number) {
        Ok(_) => {
            db.save().unwrap();
            StatusCode::CREATED
        }
        Err(err) => {
            println!("{}", err.message);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn handle_get_comments(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> (StatusCode, Json<FileComments>) {
    println!("GET Request received: {:?}", query);
    let db = DB::new(state.db_path).unwrap();
    let file_name = query.get(&"file_name".to_string()).unwrap();
    let file_name = file_name.replace(&state.repo_path, &"".to_string());

    match db.get_file_comments(&file_name) {
        Some(comments) => (StatusCode::CREATED, Json(comments)),
        None => (
            StatusCode::BAD_REQUEST,
            Json(FileComments(HashMap::default())),
        ),
    }
}
