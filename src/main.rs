use deadman::{init, init_db, start_server, tasks::start_jobs, telegram::start_bot};
use teloxide::Bot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init().await?;
    let pool = init_db().await?;
    let bot = Bot::from_env();
    start_jobs(bot.clone(), pool.clone()).await;
    start_bot(bot, pool.clone()).await?;
    start_server(pool).await?;

    Ok(())
}
