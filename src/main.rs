mod bot;
mod token;

use bot::Bot;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

const POLL_INTERVAL: u64 = 2; // every 5 minutes (2 seconds for testing)

type SharedProcessMap = Arc<Mutex<HashMap<String, Child>>>;

#[tokio::main]
async fn main() {
    let token = token::load_token();
    let mut bot = Bot::new(token);
    let running_bots: SharedProcessMap = Arc::new(Mutex::new(HashMap::new()));

    println!("Commander Bot is running...");

    loop {
        if let Some(message) = bot.update().await {
            let text = message.text.trim();
            let chat_id = message.chat_id;
            handle_message(text, chat_id, &mut bot, Arc::clone(&running_bots)).await;
        }
        sleep(Duration::from_secs(POLL_INTERVAL)).await;
    }
}

async fn handle_message(
    text: &str,
    chat_id: i64,
    bot: &mut Bot,
    running_bots: SharedProcessMap,
) {
    let mut parts = text.trim().split_whitespace();
    let command = parts.next().unwrap_or("");
    let bot_name = parts.next().unwrap_or("");

    match command {
        "/run" => {
            if bot_name.is_empty() {
                bot.send_message(chat_id, "Usage: /run <bot-name>").await;
                return;
            }

            let mut map = running_bots.lock().unwrap();
            if map.contains_key(bot_name) {
                bot.send_message(chat_id, &format!("Bot '{}' is already running.", bot_name)).await;
                return;
            }

            let path = format!("bots/{bot_name}");
            match Command::new("cargo")
                .arg("run")
                .arg("--manifest-path")
                .arg("Cargo.toml")
                .current_dir(&path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => {
                    map.insert(bot_name.to_string(), child);
                    bot.send_message(chat_id, &format!("Started bot '{}'.", bot_name)).await;
                }
                Err(e) => {
                    bot.send_message(chat_id, &format!("Failed to start bot '{}': {}", bot_name, e)).await;
                }
            }
        }
        "/stop" => {
            let mut map = running_bots.lock().unwrap();
            if let Some(mut child) = map.remove(bot_name) {
                let _ = child.kill();
                bot.send_message(chat_id, &format!("Stopped bot '{}'.", bot_name)).await;
            } else {
                bot.send_message(chat_id, &format!("Bot '{}' is not running.", bot_name)).await;
            }
        }
        _ => {
            bot.send_message(chat_id, "Unknown command. Use /run <bot-name> or /stop <bot-name>.").await;
        }
    }
}
