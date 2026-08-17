use std::sync::Arc;
use teloxide::prelude::*;
use tracing::info;

mod ai;
mod admin_handlers;
mod config;
mod db;
mod esoterics;
mod handlers;
mod keyboards;
mod models;
mod offer;
mod states;

use admin_handlers::{handle_admin_callback, handle_admin_message};
use ai::AiClient;
use config::Config;
use db::Db;
use handlers::{handle_callback, handle_message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Загрузка конфигурации и переменных окружения
    let config = Config::from_env()?;
    config.init_logging();

    info!("Starting Sacred Divination AI Telegram Bots Engine...");
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
    let config_arc = Arc::new(config.clone());

    // 4. Инициализация 1-го бота: БОТ ДЛЯ ПОЛЬЗОВАТЕЛЕЙ (@arch_reality_2026_bot)
    let user_bot = Bot::new(&config.teloxide_token);
    let user_handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback));

    let mut user_dispatcher = Dispatcher::builder(user_bot, user_handler)
        .dependencies(dptree::deps![db_arc.clone(), ai_arc.clone(), config_arc.clone()])
        .enable_ctrlc_handler()
        .build();

    // 5. Инициализация 2-го бота: БОТ ДЛЯ НАСТРОЕК И АДМИНКИ (@arch_settings_bot)
    let admin_bot_token = std::env::var("ADMIN_BOT_TOKEN").unwrap_or_else(|_| config.teloxide_token.clone());
    let admin_bot = Bot::new(&admin_bot_token);
    let admin_handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_admin_message))
        .branch(Update::filter_callback_query().endpoint(handle_admin_callback));

    let mut admin_dispatcher = Dispatcher::builder(admin_bot, admin_handler)
        .dependencies(dptree::deps![db_arc.clone(), ai_arc.clone(), config_arc.clone()])
        .enable_ctrlc_handler()
        .build();

    info!("🚀 ОБОИХ БОТОВ УСПЕШНО ЗАПУСКАЕМ ОДНОВРЕМЕННО:");
    info!("1. Бот пользователей: @arch_reality_2026_bot");
    info!("2. Бот настроек/админки: @arch_settings_bot");

    // 6. Одновременный запуск двух ботов через tokio::join
    tokio::select! {
        _ = user_dispatcher.dispatch() => info!("User bot stopped"),
        _ = admin_dispatcher.dispatch() => info!("Admin bot stopped"),
    }

    info!("Bots shutdown complete.");
    Ok(())
}
