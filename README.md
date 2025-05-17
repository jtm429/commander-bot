# Commander Bot

Hey! Hey, you! You like bots, right? Course you do. You're a bot guy. And lemme tell ya—I got **the** bot for your bots. You ever wish there was a bot that could wrangle your other bots? Keep 'em in line? Boot 'em up, shut 'em down? Fuggedaboutit. You're lookin' at *Commander Bot*.

### 🚀 What’s This Thing Do?

Commander Bot is your **Telegram-powered bot manager**. You talk to Commander Bot on Telegram, and it talks to the bots in your `bots/` folder. Like a little union boss for your digital minions.

### 🧰 Setup (It’s Easy, I Swear)

1. **Get your Telegram API token** from the BotFather.
2. **Put that bad boy in a `.token` file** in the root directory. Just the token, no quotes, no spaces.
3. **Clone or add your bots** into the `bots/` folder. Each one should be its own little Rust project.
4. **Important, listen here:** You gotta run `cargo build --release` inside each bot’s folder *before* you run Commander Bot. Otherwise they’ll run like wet spaghetti. Build ‘em first, capisce?

### 🕹️ How to Use

Start Commander Bot and message it on Telegram:

* `/run echo-bot` — Starts the bot in `bots/echo-bot`
* `/stop echo-bot` — Tucks it back in for a nap

You add whatever bots you want. You don’t gotta tell Commander in advance. He’s flexible.

### 🛠️ Requirements

* Rust (you want stable builds, don’t come in here with that nightly nonsense)
* A `.token` file for Commander, and `.token` files inside each bot’s folder
* Bots must have a `Cargo.toml` and an entry point at `src/main.rs`

### 🧼 Cleanup

Commander Bot only runs one copy of each bot at a time. You want two copies? You name it different. I don’t make the rules (actually I do, but still).

### ❗ One More Thing

Each bot is run from **its own folder**, so its `.token` file won’t get mixed up with Commander’s. You keep ‘em separate, you keep ‘em clean.

Now go on. Get outta here and manage your bots like a professional.
