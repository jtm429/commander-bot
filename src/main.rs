mod bot;
mod token;

use bot::Bot;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::fs;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

const POLL_INTERVAL: u64 = 300;       // 5 minutes
const TEMP_POLL_INTERVAL: u64 = 5;    // 5 seconds
const FAST_POLL_CYCLES: u16 = 120;      // How many fast cycles to run after a message

type SharedProcessMap = Arc<Mutex<HashMap<String, Child>>>;

#[tokio::main]
async fn main() {
    let token = token::load_token();
    let mut bot = Bot::new(token);

    // Optional: set Telegram bot commands
    bot.set_command_menu(vec![
        ("run", "Start a bot in the bots/ folder"),
        ("stop", "Stop a running bot"),
    ])
    .await;

    let running_bots: SharedProcessMap = Arc::new(Mutex::new(HashMap::new()));

    println!("Commander Bot is running...");

    let mut fast_poll_counter: u16 = 0;

    loop {
        if let Some(message) = bot.update().await {
            let text = message.text.trim();
            let chat_id = message.chat_id;
            handle_message(text, chat_id, &mut bot, Arc::clone(&running_bots)).await;

            fast_poll_counter = FAST_POLL_CYCLES; // trigger faster polling
        }

        let wait = if fast_poll_counter > 0 {
            fast_poll_counter -= 1;
            TEMP_POLL_INTERVAL
        } else {
            POLL_INTERVAL
        };

        sleep(Duration::from_secs(wait)).await;
    }
}
//generates the list of bots you can run
fn list_bot_names() -> Vec<String> {
    match fs::read_dir("bots") {
        Ok(entries) => entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_dir() {
                    path.file_name()?.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => vec![],
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
        bot.send_message_with_buttons(
            chat_id,
            "Which bot do you want to run?",
            list_bot_names().iter().map(String::as_str).collect(),
        )
        .await;

        if let Some((chat_id, choice)) = bot.await_callback_once().await {
            println!("User picked: {}", choice);
            try_start_bot(&choice, chat_id, bot, Arc::clone(&running_bots)).await;
        }

        return;
    }

    try_start_bot(bot_name, chat_id, bot, Arc::clone(&running_bots)).await;
}
"/stop" => {
    if bot_name.is_empty() {
        let map = running_bots.lock().unwrap();
        let running: Vec<String> = map.keys().cloned().collect();
        drop(map); // Release lock before awaiting anything

        if running.is_empty() {
            bot.send_message(chat_id, "No bots are currently running.").await;
            return;
        }

        bot.send_message_with_buttons(
            chat_id,
            "Which bot do you want to stop?",
            running.iter().map(String::as_str).collect(),
        )
        .await;

        if let Some((chat_id, choice)) = bot.await_callback_once().await {
            try_stop_bot(&choice, chat_id, bot, Arc::clone(&running_bots)).await;
        }

        return;
    }

    try_stop_bot(bot_name, chat_id, bot, Arc::clone(&running_bots)).await;
}
        _ => {
            bot.send_message(
                chat_id,
                "Unknown command. Use /run <bot-name> or /stop <bot-name>.",
            )
            .await;
        }
    }
}
async fn try_start_bot(
    bot_name: &str,
    chat_id: i64,
    bot: &mut Bot,
    running_bots: SharedProcessMap,
) {
    let mut map = running_bots.lock().unwrap();
    if map.contains_key(bot_name) {
        bot.send_message(chat_id, &format!("Bot '{}' is already running.", bot_name))
            .await;
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
            bot.send_message(
                chat_id,
                &format!("Failed to start bot '{}': {}", bot_name, e),
            )
            .await;
        }
    }
}

async fn try_stop_bot(
    bot_name: &str,
    chat_id: i64,
    bot: &mut Bot,
    running_bots: SharedProcessMap,
) {
    let mut map = running_bots.lock().unwrap();
    if let Some(mut child) = map.remove(bot_name) {
        let _ = child.kill();
        bot.send_message(chat_id, &format!("Stopped bot '{}'.", bot_name)).await;
    } else {
        bot.send_message(chat_id, &format!("Bot '{}' is not running.", bot_name)).await;
    }
}