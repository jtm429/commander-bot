use reqwest::Client;
use std::fs;

/// Represents a simplified incoming Telegram message.
pub struct IncomingMessage {
    pub text: String,
    pub chat_id: i64,
    pub user_id: i64,
}

/// A lightweight Telegram bot client that polls the Telegram API.
pub struct Bot {
    client: Client,             // HTTP client to send requests to Telegram
    api_url: String,            // Base URL of the Telegram API with token
    offset: i64,                // Offset for polling to avoid duplicate messages
    allowed_users: Vec<i64>,   // List of user IDs allowed to interact with the bot
}

impl Bot {
    /// Creates a new bot with the given token and loads allowed users from file.
    pub fn new(token: String) -> Self {
        let api_url = format!("https://api.telegram.org/bot{}", token);
        let allowed_users = Self::load_allowed_users("allowed_users.txt");

        Self {
            client: Client::new(),
            api_url,
            offset: 0,
            allowed_users,
        }
    }

    /// Loads user IDs from `allowed_users.txt`, one ID per line.
    fn load_allowed_users(path: &str) -> Vec<i64> {
        fs::read_to_string(path)
            .expect("Failed to read allowed_users.txt")
            .lines()
            .filter_map(|line| line.trim().parse::<i64>().ok())
            .collect()
    }

    /// Polls the Telegram API for new messages and returns the first allowed one.
    pub async fn update(&mut self) -> Option<IncomingMessage> {
        let url = format!("{}/getUpdates?timeout=30&offset={}", self.api_url, self.offset);
        let response = self.client.get(&url).send().await.ok()?.text().await.ok()?;

        println!("Raw Telegram update:\n{}", response);
        let response_json: serde_json::Value = serde_json::from_str(&response).ok()?;

        for update in response_json["result"].as_array()? {
            self.offset = update["update_id"].as_i64()? + 1;

            let message = update.get("message")?;
            let text = message.get("text")?.as_str()?.to_string();
            let chat_id = message.get("chat")?.get("id")?.as_i64()?;
            let user_id = message.get("from")?.get("id")?.as_i64()?;

            if self.allowed_users.contains(&user_id) {
                return Some(IncomingMessage { text, chat_id, user_id });
            } else {
                println!("Blocked message from unauthorized user: {}", user_id);
            }
        }

        None
    }

    /// Sends a plain text message to a Telegram chat.
    pub async fn send_message(&self, chat_id: i64, text: &str) {
        let _ = self.client
            .post(format!("{}/sendMessage", self.api_url))
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": text,
            }))
            .send()
            .await;
    }

    pub async fn send_message_with_buttons(&self, chat_id: i64, text: &str, options: Vec<&str>) {
    let buttons: Vec<serde_json::Value> = options
        .into_iter()
        .map(|label| {
            serde_json::json!({
                "text": label,
                "callback_data": label
            })
        })
        .collect();

    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "reply_markup": {
            "inline_keyboard": [ buttons ]
        }
    });

    let _ = self.client
        .post(format!("{}/sendMessage", self.api_url))
        .json(&payload)
        .send()
        .await;
    }
    //await button presses
    pub async fn await_callback_once(&mut self) -> Option<(i64, String)> {
        loop {
            let url = format!("{}/getUpdates?timeout=30&offset={}", self.api_url, self.offset);
            let response = self.client.get(&url).send().await.ok()?.text().await.ok()?;
            let response_json: serde_json::Value = serde_json::from_str(&response).ok()?;
    
            for update in response_json["result"].as_array()? {
                self.offset = update["update_id"].as_i64()? + 1;
    
                if let Some(query) = update.get("callback_query") {
                    let data = query.get("data")?.as_str()?.to_string();
                    let chat_id = query.get("message")?.get("chat")?.get("id")?.as_i64()?;
    
                    return Some((chat_id, data));
                }
            }
    
            // Sleep briefly to avoid hammering the API if no updates
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    /// (Optional) Sets the bot's command menu shown in Telegram.
    pub async fn set_command_menu(&self, commands: Vec<(&str, &str)>) {
        if commands.is_empty() {
            return; // silently skip if empty
        }

        let url = format!("{}/setMyCommands", self.api_url);
        let payload = serde_json::json!({
            "commands": commands
                .into_iter()
                .map(|(cmd, desc)| {
                    serde_json::json!({ "command": cmd, "description": desc })
                })
                .collect::<Vec<_>>()
        });

        let client = Client::new();
        match client.post(&url).json(&payload).send().await {
            Ok(response) => {
                if let Ok(text) = response.text().await {
                    println!("[Commander] Telegram setMyCommands response: {}", text);
                }
            }
            Err(e) => {
                eprintln!("[Commander] Failed to set command menu: {}", e);
            }
        }
    }
}