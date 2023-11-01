use chrono::DateTime;
use sqlx::SqlitePool;
use teloxide::{
    dispatching::{dialogue, UpdateFilterExt, UpdateHandler},
    dptree,
    prelude::{Dialogue, Dispatcher},
    requests::Requester,
    types::{ChatId, Message, Recipient, Update},
    utils::command::BotCommands,
    Bot,
};

use crate::config::NOTIFY_DELAY_MINUTES;

pub async fn maybe_notify(bot: Bot, pool: SqlitePool) -> anyhow::Result<()> {
    tracing::debug!("Maybe notify");
    let users = sqlx::query!(
        r#" 
SELECT
	user_id,
    name,
	telegram_id,
    last_call
FROM
	user
WHERE
	(last_call + interval_minutes * 60 ) < CAST(strftime('%s', 'now') as INTEGER)"#
    )
    .fetch_all(&pool)
    .await?;
    tracing::info!("Users {:?} should be notified", users);
    for user in users {
        let _ = bot
            .send_message(
                Recipient::Id(ChatId(user.telegram_id)),
                format!(
                    "Check if {} is still alive. Last message was sent at {}",
                    user.name,
                    DateTime::from_timestamp(user.last_call, 0)
                        .unwrap()
                        .to_rfc3339()
                ),
            )
            .await?;
    }
    Ok(())
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveFullName,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start the purchase procedure.")]
    Start,
    #[command(description = "cancel the purchase procedure.")]
    Cancel,
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveFullName].endpoint(receive_full_name))
        .branch(dptree::endpoint(invalid_state));

    dialogue::enter::<Update, dialogue::InMemStorage<State>, State, _>().branch(message_handler)
}

pub async fn start_bot(bot: Bot, pool: SqlitePool) -> anyhow::Result<()> {
    tokio::spawn(async move {
        Dispatcher::builder(bot, schema())
            .dependencies(dptree::deps![dialogue::InMemStorage::<State>::new(), pool])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });
    Ok(())
}

type MyDialogue = Dialogue<State, dialogue::InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start! What's your full name?")
        .await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}
async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}
async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Cancelling the dialogue.")
        .await?;
    dialogue.exit().await?;
    Ok(())
}
async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Type /help to see the usage.",
    )
    .await?;
    Ok(())
}
async fn receive_full_name(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: SqlitePool,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(name) => {
            let id = uuid::Uuid::new_v4();
            let id = id.hyphenated().to_string();
            let now = chrono::Utc::now().timestamp();
            let res = sqlx::query!(
                r#"INSERT INTO user
                (user_id, name, telegram_id, last_call, interval_minutes) 
                VALUES
                ($1, $2, $3, $4, $5)"#,
                id,
                name,
                msg.chat.id.0,
                now,
                NOTIFY_DELAY_MINUTES,
            )
            .execute(&pool)
            .await;
            match res {
                Ok(_) => {
                    bot.send_message(msg.chat.id, format!("You are registered with id '{}'", id))
                        .await?;
                }
                Err(e) => {
                    tracing::error!("Sql issue {:?}", e);
                    bot.send_message(msg.chat.id, format!("sql issue")).await?;
                }
            }

            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me your name.")
                .await?;
        }
    }
    Ok(())
}
