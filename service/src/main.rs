use auditor::{
    db::DB, get_review_state, git::Git, transform_review_state, update_review_state, FileComments,
    StoredReviewForFile, UpdateReviewState,
};
use axum::{
    extract::{Query, State},
    http::{Request, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, net::SocketAddr, time::Duration};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{info_span, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const REPO_PATH_ENV: &str = "REPO_PATH";
const DB_PATH_ENV: &str = "DB_PATH";

#[derive(Clone)]
pub struct AppState {
    repo_path: String,
    db_path: String,
}

#[derive(Deserialize)]
pub struct Transform {
    file_name: String,
}

#[derive(Serialize)]
pub struct ReviewState {
    reviewed: Vec<(usize, usize)>,
    modified: Vec<(usize, usize)>,
    ignored: Vec<(usize, usize)>,
}

#[derive(Deserialize, Debug)]
pub struct CreateComment {
    file_name: String,
    line_number: usize,
    body: String,
    author: String,
}

// #[derive(Deserialize)]
// pub struct UpdateComment {
//     line_number: usize,
//     comment_id: String,
//     body: String,
//     author: String,
// }

#[derive(Deserialize, Debug)]
pub struct DeleteComment {
    file_name: String,
    line_number: usize,
    comment_id: String,
}

impl ReviewState {
    fn default() -> Self {
        Self {
            ignored: vec![],
            modified: vec![],
            reviewed: vec![],
        }
    }
}

impl From<StoredReviewForFile> for ReviewState {
    fn from(state: StoredReviewForFile) -> Self {
        Self {
            reviewed: state
                .reviewed
                .iter()
                .map(|range| (*range.start(), *range.end()))
                .collect(),
            modified: state
                .modified
                .iter()
                .map(|range| (*range.start(), *range.end()))
                .collect(),
            ignored: state
                .ignored
                .iter()
                .map(|range| (*range.start(), *range.end()))
                .collect(),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "auditor=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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
        .route("/transform", post(handle_transform_review_state))
        .route("/comments", post(handle_create_comment))
        .route("/comments", get(handle_get_comments))
        .route("/comments", delete(handle_delete_comment))
        //.route("/comments/:comment_id", put(handle_update_comment))
        .with_state(app_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let path = request.uri().to_string();

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        path,
                    )
                })
                .on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        tracing::debug!("error: {:?}", error);
                    },
                ),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Send requests to /reviews, /transform, and /comments endpoints"
}

async fn handle_get_review_state(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> (StatusCode, Json<ReviewState>) {
    let db = DB::new(state.db_path).unwrap();
    let file_name = query.get(&"file_name".to_string());
    if file_name.is_none() {
        return (StatusCode::BAD_REQUEST, Json(ReviewState::default()));
    }
    let file_name = file_name.unwrap().replace(&state.repo_path, "");
    let file_name = file_name.replace(&state.repo_path, "");
    match get_review_state(&file_name, &db) {
        Ok(state) => (StatusCode::CREATED, Json(state.into())),
        Err(err) => {
            tracing::error!("{}", err.message);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ReviewState::default()),
            )
        }
    }
}

async fn handle_transform_review_state(
    State(state): State<AppState>,
    Json(payload): Json<Transform>,
) -> (StatusCode, Json<ReviewState>) {
    let git = Git::new(&state.repo_path).unwrap();
    let mut db = DB::new(state.db_path).unwrap();
    let file_name = payload.file_name;
    let file_name = file_name.replace(&state.repo_path, "");
    match transform_review_state(&file_name, &mut db, &git) {
        Ok(state) => {
            db.save().unwrap();
            (StatusCode::CREATED, Json(state.into()))
        }
        Err(err) => {
            tracing::error!("{}", err.message);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ReviewState::default()),
            )
        }
    }
}

async fn handle_update_review_state(
    State(state): State<AppState>,
    Json(payload): Json<UpdateReviewState>,
) -> StatusCode {
    let git = Git::new(&state.repo_path).unwrap();
    let mut db = DB::new(state.db_path).unwrap();
    let mut payload = payload;
    payload.file_name = payload.file_name.replace(&state.repo_path, "");
    match update_review_state(payload, &mut db, &git) {
        Ok(_) => {
            print!("Saving");
            db.save().unwrap();
            StatusCode::CREATED
        }
        Err(err) => {
            tracing::error!("{}", err.message);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn handle_create_comment(
    State(state): State<AppState>,
    Json(payload): Json<CreateComment>,
) -> (StatusCode, Json<String>) {
    let mut db = DB::new(state.db_path).unwrap();
    let file_name = payload.file_name.replace(&state.repo_path, "");
    match db.add_new_comment(file_name, payload.line_number, payload.body, payload.author) {
        Ok(new_comment_id) => {
            db.save().unwrap();
            (StatusCode::CREATED, Json(new_comment_id))
        }
        Err(err) => {
            tracing::error!("{}", err.message);
            (StatusCode::BAD_REQUEST, Json("".to_string()))
        }
    }
}

// async fn handle_update_comment(
//     State(state): State<AppState>,
//     Path(file_name): Path<String>,
//     Json(payload): Json<UpdateComment>,
// ) -> StatusCode {
//     let mut db = DB::new(state.db_path).unwrap();
//     match db.update_comment(
//         file_name,
//         payload.comment_id,
//         payload.line_number,
//         payload.body,
//         payload.author,
//     ) {
//         Ok(_) => StatusCode::CREATED,
//         Err(err) => {
//             StatusCode::BAD_REQUEST
//         }
//     }
// }

async fn handle_delete_comment(
    State(state): State<AppState>,
    Json(payload): Json<DeleteComment>,
) -> StatusCode {
    let mut db = DB::new(state.db_path).unwrap();
    let file_name = payload.file_name.replace(&state.repo_path, "");
    match db.delete_comment(file_name, payload.comment_id, payload.line_number) {
        Ok(_) => {
            db.save().unwrap();
            StatusCode::CREATED
        }
        Err(err) => {
            tracing::error!("{}", err.message);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn handle_get_comments(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> (StatusCode, Json<FileComments>) {
    let db = DB::new(state.db_path).unwrap();
    let file_name = query.get(&"file_name".to_string()).unwrap();
    let file_name = file_name.replace(&state.repo_path, "");

    match db.get_file_comments(&file_name) {
        Some(comments) => (StatusCode::CREATED, Json(comments)),
        None => (
            StatusCode::BAD_REQUEST,
            Json(FileComments(HashMap::default())),
        ),
    }
}
