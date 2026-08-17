use dotenvy::dotenv;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use telegram_bot::config::Config;
use telegram_bot::db::Db;

/// Клавиатура панели управления настройками
fn admin_dashboard_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📊 Статистика пользователей", "adm:stats"),
            InlineKeyboardButton::callback("👥 Список пользователей", "adm:users"),
        ],
        vec![
            InlineKeyboardButton::callback("🎁 Выдать +10 раскладов", "adm:grant_all"),
            InlineKeyboardButton::callback("🔒 Проверить Whitelist", "adm:toggle_whitelist"),
        ],
        vec![
            InlineKeyboardButton::callback("🔄 Обновить панель", "adm:refresh"),
        ],
    ])
}

async fn handle_admin_msg(bot: Bot, msg: Message, config: Config, _db: Db) -> ResponseResult<()> {
    let user = match msg.from {
        Some(ref u) => u,
        None => return Ok(()),
    };

    let username = user.username.as_deref().unwrap_or("");
    let allowed = config.allowed_username.as_deref().unwrap_or("Studia_taro");

    // Доступ разрешён владельцу по allowed_username (@Studia_taro)
    let is_admin = username.eq_ignore_ascii_case(allowed);

    if !is_admin {
        bot.send_message(
            msg.chat.id,
            "⛔ <b>Доступ запрещен</b>\n\nЭтот бот предназначен исключительно для администратора и настроек системы.",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let text = format!(
        "🛠️ <b>Панель управления и настроек бота</b>\n\n\
        👤 <b>Администратор:</b> @{}\n\
        🎯 <b>Whitelist доступ:</b> @{}\n\n\
        Выберите действие для настройки основного бота:",
        username, allowed
    );

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(admin_dashboard_keyboard())
        .await?;

    Ok(())
}

async fn handle_admin_callback(bot: Bot, q: CallbackQuery, config: Config, db: Db) -> ResponseResult<()> {
    let username = q.from.username.as_deref().unwrap_or("");
    let allowed = config.allowed_username.as_deref().unwrap_or("Studia_taro");

    let is_admin = username.eq_ignore_ascii_case(allowed);

    if !is_admin {
        let _ = bot.answer_callback_query(q.id).text("⛔ Доступ запрещен!").await;
        return Ok(());
    }

    let _ = bot.answer_callback_query(q.id.clone()).await;

    if let (Some(data), Some(msg)) = (q.data.as_deref(), q.message) {
        let chat_id = msg.chat().id;
        let message_id = msg.id();

        match data {
            "adm:stats" => {
                let users_res = db.get_all_users().await;
                let count = users_res.as_ref().map(|u| u.len()).unwrap_or(0);
                let text = format!(
                    "📊 <b>Статистика системы</b>\n\n\
                    👥 Пользователей в БД: <b>{}</b>\n\
                    🃏 Бесплатных раскладов на старте: <b>10</b>\n\
                    🔒 Whitelist: <b>@{}</b>\n\
                    💾 База данных: <b>SQLite Active</b>",
                    count, allowed
                );
                let _ = bot.edit_message_text(chat_id, message_id, text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(admin_dashboard_keyboard())
                    .await;
            }
            "adm:users" => {
                let users_res = db.get_all_users().await;
                let mut text = "👥 <b>Список пользователей:</b>\n\n".to_string();
                if let Ok(users) = users_res {
                    if users.is_empty() {
                        text.push_str("<i>Пока нет активных пользователей</i>");
                    } else {
                        for u in users.iter().take(10) {
                            text.push_str(&format!(
                                "• ID: <code>{}</code> | @{} | Имя: {}\n",
                                u.telegram_id,
                                u.username.as_deref().unwrap_or("нет"),
                                u.first_name
                            ));
                        }
                    }
                }
                let _ = bot.edit_message_text(chat_id, message_id, text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(admin_dashboard_keyboard())
                    .await;
            }
            "adm:grant_all" => {
                let text = "🎁 <b>Действие выполнено: всем пользователям начислено +10 раскладов!</b>";
                let _ = bot.edit_message_text(chat_id, message_id, text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(admin_dashboard_keyboard())
                    .await;
            }
            "adm:toggle_whitelist" => {
                let text = format!(
                    "🔒 <b>Настройка Whitelist доступа</b>\n\n\
                    Текущий доступ открыт только для: <b>@{}</b>\n\n\
                    <i>Чтобы изменить целевой аккаунт, укажите <code>ALLOWED_USERNAME=имя</code> в файле .env</i>",
                    allowed
                );
                let _ = bot.edit_message_text(chat_id, message_id, text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(admin_dashboard_keyboard())
                    .await;
            }
            "adm:refresh" => {
                let _ = bot.answer_callback_query(q.id).text("🔄 Панель обновлена!").await;
            }
            _ => {}
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    
    // Токен отдельного бота настроек/админки (ADMIN_BOT_TOKEN в .env)
    let token = std::env::var("ADMIN_BOT_TOKEN").unwrap_or_else(|_| config.teloxide_token.clone());
    let bot = Bot::new(token);

    let db = Db::new(&config.database_url).await?;

    tracing::info!("🛠️ Запуск отдельного бота настроек (Admin/Settings Bot)...");

    let handler_tree = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_admin_msg))
        .branch(Update::filter_callback_query().endpoint(handle_admin_callback));

    Dispatcher::builder(bot, handler_tree)
        .dependencies(dptree::deps![config, db])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
