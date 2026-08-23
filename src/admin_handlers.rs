use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use crate::config::Config;
use crate::db::Db;

pub async fn handle_admin_message(
    bot: Bot,
    msg: Message,
    db: Arc<Db>,
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

    let text_content = msg.text().unwrap_or("").trim();

    // Проверка текстовых команд от админа, например:
    // /grant_premium <user_id> <days>
    // /grant_energy <user_id> <amount>
    if text_content.starts_with("/grant_premium") {
        let parts: Vec<&str> = text_content.split_whitespace().collect();
        if parts.len() == 3 {
            if let (Ok(uid), Ok(days)) = (parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
                if let Ok(_) = db.set_user_premium(uid, days).await {
                    let _ = bot.send_message(
                        msg.chat.id,
                        format!("✅ Пользователю <code>{}</code> успешно выдан <b>Премиум на {} дней</b>!", uid, days)
                    ).parse_mode(ParseMode::Html).await;
                    return Ok(());
                }
            }
        }
        let _ = bot.send_message(msg.chat.id, "Формат: <code>/grant_premium &lt;user_id&gt; &lt;days&gt;</code>\nПример: <code>/grant_premium 123456789 30</code>").parse_mode(ParseMode::Html).await;
        return Ok(());
    }

    if text_content.starts_with("/grant_energy") {
        let parts: Vec<&str> = text_content.split_whitespace().collect();
        if parts.len() == 3 {
            if let (Ok(uid), Ok(amount)) = (parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
                if let Ok(_) = db.add_user_energy(uid, amount).await {
                    let _ = bot.send_message(
                        msg.chat.id,
                        format!("✅ Пользователю <code>{}</code> успешно начислено <b>+{} раскладов энергии</b>!", uid, amount)
                    ).parse_mode(ParseMode::Html).await;
                    return Ok(());
                }
            }
        }
        let _ = bot.send_message(msg.chat.id, "Формат: <code>/grant_energy &lt;user_id&gt; &lt;amount&gt;</code>\nПример: <code>/grant_energy 123456789 5</code>").parse_mode(ParseMode::Html).await;
        return Ok(());
    }

    let text = "⚙️ <b>Панель управления и настроек бота</b>\n\n\
        Добро пожаловать в центр управления Оракулом!\n\
        Здесь вы можете управлять Премиум-подписками, балансом энергии и смотреть аналитику.";

    let kb = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📊 Статистика и аналитика", "adm:stats"),
            InlineKeyboardButton::callback("👥 Список пользователей", "adm:users"),
        ],
        vec![
            InlineKeyboardButton::callback("🎁 Всем +5 раскладов", "adm:give_all_energy_5"),
            InlineKeyboardButton::callback("⭐ Выдать Премиум", "adm:manage_premium"),
        ],
        vec![
            InlineKeyboardButton::callback("🔒 Whitelist статус", "adm:whitelist"),
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
            let premium_count = users.iter().filter(|u| u.is_premium).count();
            let total_energy: i64 = users.iter().map(|u| u.energy_balance).sum();

            let text = format!(
                "📊 <b>Статистика и аналитика системы:</b>\n\n\
                👤 Всего пользователей: <b>{}</b>\n\
                ⭐ Активных Премиум-подписок: <b>{}</b>\n\
                ⚡ Суммарный баланс доп. энергии: <b>{}</b>\n\
                🗄 База данных: <b>SQLite (активна)</b>\n\
                🤖 ИИ Модель: <b>{}</b>\n\
                🔒 Whitelist: <b>@{}</b>",
                users.len(),
                premium_count,
                total_energy,
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
                list_text.push_str("Нет зарегистрированных пользователей.\n");
            } else {
                for (idx, u) in users.iter().enumerate().take(15) {
                    let premium_mark = if u.is_premium { " ⭐[ПРЕМИУМ]" } else { "" };
                    list_text.push_str(&format!(
                        "{}. <b>{}</b> (ID: <code>{}</code>) | ⚡ Энергия: {}{}\n",
                        idx + 1,
                        u.username.as_deref().unwrap_or(&u.first_name),
                        u.telegram_id,
                        u.energy_balance,
                        premium_mark
                    ));
                }
            }
            list_text.push_str("\n<i>Для выдачи премиума или раскладов отправьте команду:</i>\n<code>/grant_premium &lt;ID&gt; 30</code> или <code>/grant_energy &lt;ID&gt; 5</code>");

            let kb = InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback("🔙 Назад в меню", "adm:main_menu")]
            ]);
            let _ = bot.edit_message_text(chat_id, message_id, list_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(kb)
                .await;
        }
        "adm:give_all_energy_5" => {
            let users = db.get_all_users().await.unwrap_or_default();
            let count = users.len();
            for u in &users {
                let _ = db.add_user_energy(u.telegram_id, 5).await;
            }
            let text = format!(
                "🎁 <b>Массовое начисление энергии:</b>\n\n\
                ✅ Всем <b>{}</b> пользователям успешно начислено <b>+5 раскладов</b>!\n\
                Баланс энергии обновлен в базе данных.",
                count
            );
            let kb = InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback("🔙 Назад в меню", "adm:main_menu")]
            ]);
            let _ = bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(kb)
                .await;
        }
        "adm:manage_premium" => {
            let text = "⭐ <b>Управление Премиум-подписками</b>\n\n\
                Чтобы активировать Премиум пользователю, отправьте команду в чат:\n\n\
                • <b>1 месяц (30 дней):</b> <code>/grant_premium &lt;ID&gt; 30</code>\n\
                • <b>3 месяца (90 дней):</b> <code>/grant_premium &lt;ID&gt; 90</code>\n\
                • <b>1 год (365 дней):</b> <code>/grant_premium &lt;ID&gt; 365</code>\n\n\
                • <b>Пакет +5 раскладов:</b> <code>/grant_energy &lt;ID&gt; 5</code>\n\n\
                <i>ID пользователя можно скопировать из раздела «👥 Список пользователей».</i>";
            let kb = InlineKeyboardMarkup::new(vec![
                vec![
                    InlineKeyboardButton::callback("👥 Открыть список пользователей", "adm:users"),
                ],
                vec![
                    InlineKeyboardButton::callback("🔙 Назад в меню", "adm:main_menu"),
                ],
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
                Здесь вы можете управлять Премиум-подписками, балансом энергии и смотреть аналитику.";
            let kb = InlineKeyboardMarkup::new(vec![
                vec![
                    InlineKeyboardButton::callback("📊 Статистика и аналитика", "adm:stats"),
                    InlineKeyboardButton::callback("👥 Список пользователей", "adm:users"),
                ],
                vec![
                    InlineKeyboardButton::callback("🎁 Всем +5 раскладов", "adm:give_all_energy_5"),
                    InlineKeyboardButton::callback("⭐ Выдать Премиум", "adm:manage_premium"),
                ],
                vec![
                    InlineKeyboardButton::callback("🔒 Whitelist статус", "adm:whitelist"),
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
