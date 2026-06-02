use serde::Deserialize;
use sqlx::PgPool;
use std::time::Duration;

#[derive(Deserialize, Debug)]
struct TgResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TgBotInfo {
    username: String,
}

#[derive(Deserialize, Debug)]
struct TgUpdate {
    update_id: i64,
    message: Option<TgMessage>,
}

#[derive(Deserialize, Debug)]
struct TgMessage {
    chat: TgChat,
    from: Option<TgUser>,
    text: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TgChat {
    id: i64,
}

#[derive(Deserialize, Debug)]
struct TgUser {
    id: i64,
    first_name: Option<String>,
    last_name: Option<String>,
    username: Option<String>,
}

pub async fn get_bot_username(client: &reqwest::Client, bot_token: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("https://api.telegram.org/bot{}/getMe", bot_token);
    let resp = client.get(&url).send().await?.json::<TgResponse<TgBotInfo>>().await?;
    if resp.ok {
        if let Some(info) = resp.result {
            return Ok(info.username);
        }
    }
    let err_desc = resp.description.unwrap_or_else(|| "Unknown error".to_string());
    Err(format!("Telegram error: {}", err_desc).into())
}

pub async fn run_bot_polling(
    client: reqwest::Client,
    bot_token: String,
    pool: PgPool,
) {
    let mut offset: i64 = 0;
    println!("Telegram bot polling started.");

    loop {
        // Clean up expired tokens (older than 10 minutes)
        let expire_time = chrono::Utc::now().timestamp() - 600;
        if let Err(e) = sqlx::query("DELETE FROM auth_tokens WHERE created_at < $1")
            .bind(expire_time)
            .execute(&pool)
            .await
        {
            eprintln!("Failed to clean up expired auth tokens: {}", e);
        }

        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=30",
            bot_token, offset
        );


        let response = match client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Telegram polling network error: {}. Retrying in 5 seconds...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let updates_resp = match response.json::<TgResponse<Vec<TgUpdate>>>().await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Telegram polling JSON error: {}. Retrying in 5 seconds...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        if !updates_resp.ok {
            eprintln!(
                "Telegram polling API returned ok=false: {:?}. Retrying in 5 seconds...",
                updates_resp.description
            );
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        if let Some(updates) = updates_resp.result {
            for update in updates {
                offset = update.update_id + 1;

                if let Some(message) = update.message {
                    if let Some(text) = message.text {
                        if text.starts_with("/start ") {
                            let token = text.strip_prefix("/start ").unwrap_or("").trim();
                            if !token.is_empty() {
                                let chat_id = message.chat.id;
                                let user = message.from.unwrap_or(TgUser {
                                    id: chat_id,
                                    first_name: None,
                                    last_name: None,
                                    username: None,
                                });

                                // Check and update in database
                                match handle_auth_message(&pool, token, &user).await {
                                    Ok(true) => {
                                        // Send success message
                                        let reply = "Вы успешно авторизовались на сайте! Возвращайтесь во вкладку браузера для завершения входа.";
                                        let _ = send_telegram_message(&client, &bot_token, chat_id, reply).await;
                                    }
                                    Ok(false) => {
                                        // Token not found or expired
                                        let reply = "Ошибка: неверный или устаревший код авторизации. Пожалуйста, вернитесь на сайт и запросите код заново.";
                                        let _ = send_telegram_message(&client, &bot_token, chat_id, reply).await;
                                    }
                                    Err(e) => {
                                        eprintln!("Database error processing auth token: {}", e);
                                        let reply = "Произошла внутренняя ошибка сервера. Пожалуйста, попробуйте позже.";
                                        let _ = send_telegram_message(&client, &bot_token, chat_id, reply).await;
                                    }
                                }
                            }
                        } else if text == "/start" {
                            let chat_id = message.chat.id;
                            let reply = "Привет! Этот бот используется для безопасного входа на сайт. Пожалуйста, перейдите на страницу авторизации на сайте и нажмите кнопку входа.";
                            let _ = send_telegram_message(&client, &bot_token, chat_id, reply).await;
                        }
                    }
                }
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct TokenStatusRow {
    status: String,
}

async fn handle_auth_message(
    pool: &PgPool,
    token: &str,
    user: &TgUser,
) -> Result<bool, sqlx::Error> {
    // 1. Check if token exists and is pending
    let token_exists = sqlx::query_as::<_, TokenStatusRow>(
        "SELECT status FROM auth_tokens WHERE token = $1"
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = token_exists {
        if row.status == "pending" {
            // Update token with telegram details and status
            let now = chrono::Utc::now().timestamp();
            sqlx::query(
                "UPDATE auth_tokens 
                 SET telegram_id = $1, 
                     first_name = $2, 
                     last_name = $3, 
                     username = $4, 
                     auth_date = $5, 
                     status = 'authenticated'
                 WHERE token = $6"
            )
            .bind(user.id)
            .bind(&user.first_name)
            .bind(&user.last_name)
            .bind(&user.username)
            .bind(now)
            .bind(token)
            .execute(pool)
            .await?;

            return Ok(true);
        }
    }

    Ok(false)
}


async fn send_telegram_message(
    client: &reqwest::Client,
    bot_token: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), reqwest::Error> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let params = [
        ("chat_id", chat_id.to_string()),
        ("text", text.to_string()),
    ];
    let _ = client.post(&url).form(&params).send().await?;
    Ok(())
}
