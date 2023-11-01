use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::Sqlite, Pool, SqlitePool};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init().await?;
    let pool = init_db().await?;
    start_server(pool).await?;
    Ok(())
}

async fn init() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();
    Ok(())
}

async fn init_db() -> anyhow::Result<SqlitePool> {
    let pool = Pool::<Sqlite>::connect("sqlite:deadman.sqlite")
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    Ok(pool)
}

fn get_router(pool: SqlitePool) -> anyhow::Result<Router> {
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/not_dead", post(not_dead))
        .with_state(pool);
    Ok(app)
}

async fn start_server(pool: SqlitePool) -> anyhow::Result<()> {
    let app = get_router(pool)?;
    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn not_dead(
    State(pool): State<SqlitePool>,
    Query(query): Query<NotDeadParams>,
) -> (StatusCode, Json<Message>) {
    eprintln!("{:?}", query.id);
    let now = Utc::now().timestamp();
    let ret = sqlx::query!(
        r#"UPDATE user
        SET last_call = $1
        WHERE user_id = $2
        RETURNING user_id"#,
        now,
        query.id
    )
    .fetch_all(&pool)
    .await;
    match ret {
        Err(err) => {
            tracing::error!("sqlx {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Message {
                    msg: Some("sql issue".to_string()),
                }),
            );
        }
        Ok(users) => {
            if users.is_empty() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(Message {
                        msg: Some("User not found".to_string()),
                    }),
                );
            }
        }
    }

    (StatusCode::OK, Json(Message { msg: None }))
}

#[derive(Serialize)]
struct Message {
    msg: Option<String>,
}

#[derive(Deserialize, Debug)]
struct NotDeadParams {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::axum_test::TestServer;

    #[sqlx::test(fixtures("users"))]
    async fn test_not_dead(pool: SqlitePool) {
        init().await.unwrap();
        let old_user = sqlx::query!(r#"SELECT last_call FROM user WHERE user_id = 'test_id'"#)
            .fetch_one(&pool)
            .await
            .unwrap();
        let server = TestServer::new(get_router(pool.clone()).unwrap()).unwrap();
        let ret = server
            .post("/not_dead")
            .add_query_param("id", "test_id")
            .await;
        tracing::debug!("Ret {:?}", ret);
        assert!(ret.status_code().is_success());
        let user = sqlx::query!(r#"SELECT last_call FROM user WHERE user_id = 'test_id'"#)
            .fetch_one(&pool)
            .await
            .unwrap();
        tracing::debug!("now {} < last_call {}", old_user.last_call, user.last_call);
        assert!(old_user.last_call < user.last_call);
    }
}
