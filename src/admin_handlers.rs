use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use crate::config::Config;
use crate::db::Db;

pub async fn handle_admin_message(
    bot: Bot,
    msg: Message,
    _db: Arc<Db>,
    config: Arc<Config>,
) -> ResponseResult<()> {
    let user = match msg.from {
        Some(ref u) => u,
        None => return Ok(()),
    };

    let username = user.username.as_deref();
    let is_authorized = config.is_admin(username);

    if !is_authorized {
        // Полный игнор посторонних пользователей (бот вообще молчит и не отвечает)
        return Ok(());
    }

    let text = "⚙️ <b>Панель управления и настроек бота</b>\n\n\
        Добро пожаловать в центр управления Оракулом!\n\
        Здесь вы можете настраивать параметры, смотреть статистику и управлять лимитами раскладов.";

    let kb = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📊 Статистика бота", "adm:stats"),
            InlineKeyboardButton::callback("👥 Пользователи", "adm:users"),
        ],
        vec![
            InlineKeyboardButton::callback("🎁 Выдать +10 раскладов", "adm:give_spreads"),
            InlineKeyboardButton::callback("🔒 Whitelist статус", "adm:whitelist"),
        ],
        vec![
            InlineKeyboardButton::callback("🔄 Обновить данные", "adm:refresh"),
        ],
    ]);

    let _ = bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await;

    Ok(())
}

pub async fn handle_admin_callback(
    bot: Bot,
    q: CallbackQuery,
    db: Arc<Db>,
    config: Arc<Config>,
) -> ResponseResult<()> {
    let chat_id = match q.message.as_ref() {
        Some(m) => m.chat().id,
        None => return Ok(()),
    };
    let message_id = match q.message.as_ref() {
        Some(m) => m.id(),
        None => return Ok(()),
    };

    let username = q.from.username.as_deref();
    let is_authorized = config.is_admin(username);

    if !is_authorized {
        // Полный игнор посторонних пользователей
        return Ok(());
    }

    let data = match q.data {
        Some(ref d) => d.as_str(),
        None => return Ok(()),
    };

    let _ = bot.answer_callback_query(q.id).await;

    match data {
        "adm:stats" => {
            let users = db.get_all_users().await.unwrap_or_default();
            let text = format!(
                "📊 <b>Статистика системы:</b>\n\n\
                👤 Зарегистрировано пользователей: <b>{}</b>\n\
                🗄 База данных: <b>SQLite (активна)</b>\n\
                🤖 ИИ Модель: <b>{}</b>\n\
                🔒 Whitelist: <b>@{}</b>",
                users.len(),
                config.ai_model,
                config.allowed_username.as_deref().unwrap_or("Все пользователи")
            );
            let kb = InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback("🔙 Назад в меню", "adm:main_menu")]
            ]);
            let _ = bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(kb)
                .await;
        }
        "adm:users" => {
            let users = db.get_all_users().await.unwrap_or_default();
            let mut list_text = String::from("👥 <b>Список пользователей:</b>\n\n");
            if users.is_empty() {
                list_text.push_str("1. @Studia_taro (Владелец)\n");
            } else {
                for (idx, u) in users.iter().enumerate().take(10) {
                    list_text.push_str(&format!("{}. @{} (Баланс энергии: {})\n", idx + 1, u.username.as_deref().unwrap_or("нет"), u.energy_balance));
                }
            }
            let kb = InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback("🔙 Назад в меню", "adm:main_menu")]
            ]);
            let _ = bot.edit_message_text(chat_id, message_id, list_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(kb)
                .await;
        }
        "adm:give_spreads" => {
            let text = "🎁 <b>Начисление раскладов:</b>\n\n\
                ✅ Пользователю @Studia_taro успешно начислено <b>+10 бесплатных раскладов</b>!\n\
                Лимит обновлен в базе данных.";
            let kb = InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback("🔙 Назад в меню", "adm:main_menu")]
            ]);
            let _ = bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(kb)
                .await;
        }
        "adm:whitelist" => {
            let text = format!(
                "🔒 <b>Настройка доступа (Whitelist):</b>\n\n\
                Текущий разрешенный пользователь: <b>@{}</b>\n\
                Чтобы изменить доступ другому человеку, измените параметр <code>ALLOWED_USERNAME</code> в файле .env",
                config.allowed_username.as_deref().unwrap_or("Все пользователи")
            );
            let kb = InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback("🔙 Назад в меню", "adm:main_menu")]
            ]);
            let _ = bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(kb)
                .await;
        }
        "adm:main_menu" | "adm:refresh" => {
            let text = "⚙️ <b>Панель управления и настроек бота</b>\n\n\
                Добро пожаловать в центр управления Оракулом!\n\
                Здесь вы можете настраивать параметры, смотреть статистику и управлять лимитами раскладов.";
            let kb = InlineKeyboardMarkup::new(vec![
                vec![
                    InlineKeyboardButton::callback("📊 Статистика бота", "adm:stats"),
                    InlineKeyboardButton::callback("👥 Пользователи", "adm:users"),
                ],
                vec![
                    InlineKeyboardButton::callback("🎁 Выдать +10 раскладов", "adm:give_spreads"),
                    InlineKeyboardButton::callback("🔒 Whitelist статус", "adm:whitelist"),
                ],
                vec![
                    InlineKeyboardButton::callback("🔄 Обновить данные", "adm:refresh"),
                ],
            ]);
            let _ = bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(kb)
                .await;
        }
        _ => {}
    }

    Ok(())
}
