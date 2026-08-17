use std::sync::Arc;
use teloxide::prelude::*;
use tracing::info;

mod ai;
mod config;
mod db;
mod esoterics;
mod handlers;
mod keyboards;
mod models;
mod offer;
mod states;

use ai::AiClient;
use config::Config;
use db::Db;
use handlers::{handle_callback, handle_message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Загрузка конфигурации и переменных окружения
    let config = Config::from_env()?;
    config.init_logging();

    info!("Starting Sacred Divination AI Telegram Bot...");
    config.validate()?;

    // 2. Инициализация базы данных SQLite
    let db = Db::new(&config.database_url).await?;
    info!("Database initialized successfully");

    // 3. Инициализация ИИ-клиента
    let ai_client = AiClient::new(
        &config.ai_base_url,
        &config.ai_api_key,
        &config.ai_model,
        config.ai_timeout_secs,
    );
    info!("AI Client initialized with model: {}", config.ai_model);

    let db_arc = Arc::new(db);
    let ai_arc = Arc::new(ai_client);

    // 4. Инициализация Telegram Бота
    let bot = Bot::new(&config.teloxide_token);

    // 5. Построение дерева диспетчеризации (dptree)
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .endpoint(handle_message),
        )
        .branch(
            Update::filter_callback_query()
                .endpoint(handle_callback),
        );

    info!("Bot dispatcher started. Listening for events...");

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db_arc, ai_arc])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    info!("Bot shutdown complete.");
    Ok(())
}
