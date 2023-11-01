use chrono::Duration;
use sqlx::SqlitePool;
use teloxide::Bot;

use crate::{config::CHECK_INTERVAL_MINUTES, telegram::maybe_notify};
pub async fn start_jobs(bot: Bot, pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            let pool_clone = pool.clone();
            let bot_clone = bot.clone();
            let res = tokio::spawn(async move { maybe_notify(bot_clone, pool_clone).await }).await;
            match res {
                Err(err) => {
                    tracing::error!("Error in maybe_notify job {:?}", err);
                }
                Ok(Err(err)) => {
                    tracing::error!("Error in maybe_notify job {:?}", err);
                }
                _ => {}
            }
            tokio::time::sleep(Duration::minutes(CHECK_INTERVAL_MINUTES).to_std().unwrap()).await;
        }
    });
}
