use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{ChatAction, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, ParseMode};
use tracing::error;

use crate::ai::prompts::{
    build_astrology_prompt, build_leela_prompt, build_tarot_prompt, SYSTEM_ORACLE_PROMPT,
};
use crate::ai::AiClient;
use crate::config::Config;
use crate::db::Db;
use crate::esoterics::astrology::ZODIAC_SIGNS;
use crate::esoterics::leela::LeelaGame;
use crate::esoterics::tarot::{TarotDeck, SPREAD_TYPES, TAROT_SPHERES};
use crate::keyboards::*;
use crate::offer::{legal_menu_keyboard, DETAILED_OFFER_TEXT};

/// Обработка текстовых сообщений и команд
/// ПРАВИЛО: Пользователь НЕ может вводить произвольный текст!
pub async fn handle_message(
    bot: Bot,
    msg: Message,
    db: Arc<Db>,
    config: Arc<Config>,
) -> ResponseResult<()> {
    let user = match msg.from {
        Some(ref u) => u,
        None => return Ok(()),
    };

    let user_id = user.id.0 as i64;
    let username = user.username.clone();
    let first_name = user.first_name.clone();
    let last_name = user.last_name.clone();

    // 🔒 БЕЛЫЙ СПИСОК (Whitelist): доступ разрешен указанному пользователю или админам
    if let Some(ref allowed) = config.allowed_username {
        let is_allowed = username.as_ref().map(|u| u.eq_ignore_ascii_case(allowed)).unwrap_or(false)
            || config.is_admin(username.as_deref());
        if !is_allowed {
            let access_denied_text = format!(
                "🔒 <b>Доступ ограничен</b>\n\n\
                Бот находится в режиме приватного тестирования и доступен только для <b>@{}</b>.\n\
                Для получения персонального доступа напишите администратору.",
                allowed
            );
            let _ = bot.send_message(msg.chat.id, access_denied_text)
                .parse_mode(ParseMode::Html)
                .await;
            return Ok(());
        }
    }

    // Регистрация или получение пользователя
    let db_user = match db.create_or_update_user(user_id, username.as_deref(), &first_name, last_name.as_deref()).await {
        Ok(u) => u,
        Err(e) => {
            error!("Database error getting user: {:?}", e);
            return Ok(());
        }
    };

    let text = msg.text().unwrap_or("");

    // Если оферта еще не принята — показываем аккуратное единое сообщение
    if !db_user.is_offer_accepted {
        let bot_name = format!("@{}", config.user_bot_name);
        let raw_text = DETAILED_OFFER_TEXT.replace("@Oraculum_true_bot", &bot_name);
        let offer_intro = format!(
            "✨ <b>Добро пожаловать в Сакральный Храм Оракула</b>, {}!\n\n{}",
            first_name, raw_text
        );

        let _ = bot.send_message(msg.chat.id, offer_intro)
            .parse_mode(ParseMode::Html)
            .reply_markup(crate::offer::legal_menu_keyboard(true))
            .await;
        return Ok(());
    }

    // Обработка разрешенных команд
    if text == "/admin" || text == "/panel" {
        let is_admin = username.as_deref() == Some("Studia_taro");

        if is_admin {
            let admin_kb = InlineKeyboardMarkup::new(vec![
                vec![
                    InlineKeyboardButton::callback("📊 Статистика", "admin:stats"),
                    InlineKeyboardButton::callback("🔒 Whitelist статус", "admin:whitelist"),
                ],
                vec![InlineKeyboardButton::callback("🏠 В главное меню", "nav:main")],
            ]);

            let _ = bot.send_message(
                msg.chat.id,
                "👑 <b>Панель управления Архитектора</b>\n\n\
                Здесь доступны административные функции и управление базой пользователей:",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(admin_kb)
            .await;
            return Ok(());
        }
    }

    if text == "/start" || text == "/menu" || text == "/restart" || text == "/reset" {
        // Удаляем команду пользователя /start, чтобы чат оставался идеально чистым
        let _ = bot.delete_message(msg.chat.id, msg.id).await;

        let (_, remaining) = match db.can_make_free_reading(user_id, config.max_free_lifetime_readings).await {
            Ok(res) => res,
            Err(_) => (true, 10),
        };
        let menu_text = format!(
            "✨ <b>Главное меню ORACULUM</b>\n\n\
            🔮 Добро пожаловать в пространство сакральных знаний и ИИ-Оракула, {}!\n\n\
            🎁 Доступно бесплатных раскладов: <b>{} из 10</b>\n\n\
            Выберите интересующий вас раздел:",
            db_user.first_name, remaining
        );
        let _ = bot.send_message(msg.chat.id, menu_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
        return Ok(());
    }

    if text == "/offer" {
        let _ = bot.send_message(msg.chat.id, &DETAILED_OFFER_TEXT.replace("@Oraculum_true_bot", &format!("@{}", config.user_bot_name)))
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
        return Ok(());
    }

    // ПОЛНЫЙ ЗАПРЕТ ПРОИЗВОЛЬНОГО ТЕКСТОВОГО ВВОДА:
    let _ = bot.delete_message(msg.chat.id, msg.id).await;

    let warning_text = "🔮 <i>Оракул считывает намерения через сакральные знаки и символы. Пожалуйста, используйте кнопки навигации для управления диалогом.</i>";
    let _ = bot.send_message(msg.chat.id, warning_text)
        .parse_mode(ParseMode::Html)
        .reply_markup(main_menu_keyboard())
        .await;

    Ok(())
}

/// Обработка всех Callback запросов (нажатий на Inline-кнопки)
pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    db: Arc<Db>,
    ai: Arc<AiClient>,
    config: Arc<Config>,
) -> ResponseResult<()> {
    let data = match q.data {
        Some(d) => d,
        None => return Ok(()),
    };

    let user_id = q.from.id.0 as i64;
    let username = q.from.username.clone();
    let message = match q.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = message.chat().id;
    let message_id = message.id();

    let query_id = q.id.clone();
    // 🔒 БЕЛЫЙ СПИСОК (Whitelist) для Callback Query
    if let Some(ref allowed) = config.allowed_username {
        let is_allowed = username.as_ref().map(|u| u.eq_ignore_ascii_case(allowed)).unwrap_or(false);
        if !is_allowed {
            let _ = bot.answer_callback_query(query_id).text("🔒 Доступ только для авторизованного пользователя").await;
            return Ok(());
        }
    }

    // Проверяем / создаем пользователя
    let db_user = match db.create_or_update_user(
        user_id,
        q.from.username.as_deref(),
        &q.from.first_name,
        q.from.last_name.as_deref(),
    ).await {
        Ok(u) => u,
        Err(e) => {
            error!("Database error in callback: {:?}", e);
            let _ = bot.answer_callback_query(query_id).await;
            return Ok(());
        }
    };

    // Подтверждаем получение callback

    // 1. Принятие оферты
    if data == "accept_offer" {
        let _ = db.accept_offer(user_id).await;
        let _ = bot.answer_callback_query(q.id).text("✅ Оферта успешно принята!").await;
        
        let (_, remaining) = match db.can_make_free_reading(user_id, config.max_free_lifetime_readings).await {
            Ok(res) => res,
            Err(_) => (true, 10),
        };

        let text = format!(
            "✨ <b>Врата Оракула открыты для вас, {}!</b>\n\n\
            🎁 Доступно бесплатных раскладов: <b>{}</b>\n\n\
            Выберите таинство, к которому желает обратиться ваша душа:",
            db_user.first_name, remaining
        );
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
        return Ok(());
    }

    // Если оферта еще не принята — блокируем любые колбэки кроме оферты
    if !db_user.is_offer_accepted {
        let allowed = data.starts_with("legal:") || data.starts_with("nav:offer") || data.starts_with("nav:legal") || data == "nav:main_menu";
        if !allowed {
            let _ = bot.answer_callback_query(q.id).text("Сначала необходимо принять условия оферты").await;
            return Ok(());
        }
    }

    let _ = bot.answer_callback_query(q.id).await;

    if data == "nav:main" || data == "nav:main_menu" || data == "nav:restart" {
        let (_can_read, remaining) = match db.can_make_free_reading(user_id, config.max_free_lifetime_readings).await {
            Ok(res) => res,
            Err(_) => (true, 10),
        };
        let text = format!(
            "✨ <b>Главное меню ORACULUM</b>\n\n\
            🔮 Добро пожаловать в пространство сакральных знаний и ИИ-Оракула.\n\n\
            🎁 Доступно бесплатных раскладов: <b>{} из 10</b>\n\n\
            Выберите интересующий вас раздел:",
            remaining
        );
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
        return Ok(());
    }

    // 2. Навигация по разделам
    if data == "nav:legal" {
        let text = "⚖️ <b>Правовая информация сервиса ORACULUM</b>\n\n\
            Сервис функционирует в строгом соответствии с законодательством РФ.\n\
            Выберите интересующий вас документ для ознакомления:";
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(crate::offer::legal_menu_keyboard(!db_user.is_offer_accepted))
            .await;
        return Ok(());
    }

    if data == "legal:offer" || data.starts_with("legal:offer:") {
        let part: usize = if data == "legal:offer" {
            1
        } else {
            data.strip_prefix("legal:offer:").unwrap_or("1").parse().unwrap_or(1)
        };

        let content = match part {
            1 => crate::offer::OFFER_PART_1,
            2 => crate::offer::OFFER_PART_2,
            _ => crate::offer::OFFER_PART_3,
        };

        let formatted_text = format!("📜 <b>Публичная оферта (Часть {}/3)</b>

{}", part, content);

        let _ = bot.edit_message_text(chat_id, message_id, formatted_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(crate::offer::offer_pagination_keyboard(part))
            .await;
        return Ok(());
    }

    if data == "legal:privacy" {
        let _ = bot.edit_message_text(chat_id, message_id, crate::offer::PRIVACY_POLICY_TEXT)
            .parse_mode(ParseMode::Html)
            .reply_markup(crate::offer::legal_menu_keyboard(!db_user.is_offer_accepted))
            .await;
        return Ok(());
    }

    if data == "legal:consent" {
        let _ = bot.edit_message_text(chat_id, message_id, crate::offer::CONSENT_TEXT)
            .parse_mode(ParseMode::Html)
            .reply_markup(crate::offer::legal_menu_keyboard(!db_user.is_offer_accepted))
            .await;
        return Ok(());
    }

    if data == "nav:contact_tarologist" {
        let text = "🧿 <b>Личное обращение к тарологу</b>\n\n\
            Отправьте личный вопрос тарологу. После обращения она сможет открыть диалог с вами; формат и стоимость дальнейшей консультации обсуждаются лично.\n\n\
            Стоимость первого обращения: <b>100 ₽</b>.\n\
            После оплаты вы сможете отправить сообщение. Дальнейший формат консультации и её стоимость таролог обсудит с вами лично.\n\n\
            Для связи напишите в поддержку: <b>@Studia_taro</b>";
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(support_keyboard())
            .await;
        return Ok(());
    }

    if data == "nav:tariffs" {
        let text = "💎 <b>Тарифы и подписка ORACULUM</b>\n\n\
            Пополняйте запасы энергии или активируйте безлимитный доступ к раскладам ИИ-Оракула:\n\n\
            • ⚡ <b>Пакет «5 Раскладов»</b> — 290 ₽\n\
            • 🌟 <b>Месячный безлимит «Адепт»</b> — 790 ₽ / месяц\n\
            • 💫 <b>Безлимит «Маг» (3 месяца)</b> — 1990 ₽\n\
            • 🔮 <b>Безлимит «Верховная Жрица» (год)</b> — 5990 ₽\n\n\
            <i>Оплата принимается через СБП, карты РФ и ЮMoney. Для подключения напишите в поддержку @Studia_taro</i>.";
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(tariffs_keyboard())
            .await;
        return Ok(());
    }

    // Обработка оплаты тарифов - заглушка с перенаправлением в поддержку
    if data == "pay:pack_5" || data == "pay:sub_month" || data == "pay:sub_3months" || data == "pay:sub_year" {
        let (tariff_name, price) = if data == "pay:pack_5" {
            ("Пакет «5 Раскладов»", "290 ₽")
        } else if data == "pay:sub_month" {
            ("Безлимит «Адепт» (месяц)", "790 ₽")
        } else if data == "pay:sub_3months" {
            ("Безлимит «Маг» (3 месяца)", "1990 ₽")
        } else {
            ("Безлимит «Верховная Жрица» (год)", "5990 ₽")
        };
        
        let text = format!(
            "💎 <b>Выбран тариф: {}</b>\n\n\
            Стоимость: <b>{}</b>\n\n\
            Для подключения этого тарифа пожалуйста напишите в нашу службу поддержки:\n\
            @Studia_taro\n\n\
            Укажите желаемый тариф и способ оплаты (СБП, карта РФ, ЮMoney).",
            tariff_name, price
        );
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(support_keyboard())
            .await;
        return Ok(());
    }

    if data == "nav:support" {
        let text = "🛟 <b>Служба заботы и поддержки</b>\n\n\
            Если у вас возникли сложности с оплатой, работой сервиса или персональным разбором таролога — наша служба поддержки готова вам помочь.\n\n\
            Напишите нам напрямую в Telegram: <b>@Studia_taro</b>";
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(support_keyboard())
            .await;
        return Ok(());
    }

    if data == "nav:history" {
        let history = db.get_user_history(user_id, 5).await.unwrap_or_default();
        let mut text = String::from("📚 <b>История ваших последних раскладов</b>\n\n");
        if history.is_empty() {
            text.push_str("<i>У вас пока нет сохраненных раскладов. Сделайте ваш первый расклад через главное меню!</i>");
        } else {
            for (idx, r) in history.iter().enumerate() {
                text.push_str(&format!(
                    "{}. <b>{}</b> (<i>{}</i>)\nКарты: <code>{}</code>\n📅 {}\n\n",
                    idx + 1,
                    r.topic,
                    r.reading_type,
                    r.selected_cards,
                    r.created_at
                ));
            }
        }
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
        return Ok(());
    }

    if data == "pay:tarologist_consultation" {
        // Заглушка генерации ссылки оплаты ЮKassa / ЮMoney
        let text = "🧿 <b>Оплата личного обращения к тарологу</b>\n\n\
            Сумма к оплате: <b>100 ₽</b>\n\n\
            Нажмите кнопку ниже для безопасной оплаты через ЮMoney / СБП / Карту:";
        let yoomoney_mock_url = "https://yoomoney.ru";
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(contact_tarologist_keyboard(Some(yoomoney_mock_url)))
            .await;
        return Ok(());
    }

    if data == "nav:card_of_the_day" {
        // Быстрый запуск расклада Карта Дня - Оракул открывает Аркан вашего сегодняшнего дня
        let drawn = TarotDeck::draw_cards(1);
        let (card, is_reversed) = &drawn[0];
        let orientation = if *is_reversed { " (Перевернутая)" } else { " (Прямая)" };
        
        let wait_msg = bot.send_message(chat_id, "🌙 <i>Оракул открывает Аркан вашего сегодняшнего дня...</i>")
            .parse_mode(ParseMode::Html)
            .await;

        let prompt = format!(
            "Расклад 'Карта дня'. Выпала карта: {}{}. Дай краткий, мудрый совет, предостережение и фокус внимания на день.",
            card.name, orientation
        );

        let ai_result = match ai.generate_reading(SYSTEM_ORACLE_PROMPT, &prompt).await {
            Ok(res) => res,
            Err(_) => format!("Аркан: {}{}\n\nСвет знания: Доверьтесь интуиции и наблюдайте за знаками.", card.name, orientation),
        };

        if let Ok(m) = wait_msg {
            let _ = bot.delete_message(chat_id, m.id).await;
        }

        let caption = format!(
            "🌙 <b>Оракул открывает Аркан вашего сегодняшнего дня: {}{}</b>\n\n\
            🔮 <b>Совет Оракула:</b>\n\
            {}\n\n\
            <i>Ключевые энергии: {}</i>",
            card.name,
            orientation,
            ai_result,
            card.keywords
        );

        if let Ok(url) = card.image_url.parse() {
            let res = bot.send_photo(chat_id, teloxide::types::InputFile::url(url))
                .caption(caption.clone())
                .parse_mode(ParseMode::Html)
                .reply_markup(main_menu_keyboard())
                .await;
            if res.is_err() {
                let _ = bot.send_message(chat_id, caption)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(main_menu_keyboard())
                    .await;
            }
        } else {
            let _ = bot.send_message(chat_id, caption)
                .parse_mode(ParseMode::Html)
                .reply_markup(main_menu_keyboard())
                .await;
        }
        return Ok(());
    }

    if data == "nav:main_menu" {
        
        if !db_user.is_offer_accepted {
            let bot_name = format!("@{}", config.user_bot_name);
            let raw_text = DETAILED_OFFER_TEXT.replace("@Oraculum_true_bot", &bot_name);
            let offer_intro = format!(
                "✨ <b>Добро пожаловать в Сакральный Храм Оракула</b>, {}!\n\n{}",
                db_user.first_name, raw_text
            );
            let _ = bot.send_message(chat_id, offer_intro)
                .parse_mode(ParseMode::Html)
                .reply_markup(offer_keyboard())
                .await;
            return Ok(());
        }

        let text = format!(
            "🏛 <b>Главный зал Оракула</b>\n\n\
            Ваш запас энергии: ⚡ <b>{}</b>\n\
            Выберите желаемое таинство:",
            db_user.energy_balance
        );
        
        // Попытка отредактировать сообщение (если это текст) или отправить новое
        let edit_result = bot.edit_message_text(chat_id, message_id, text.clone())
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
            
        if edit_result.is_err() {
            let _ = bot.send_message(chat_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(main_menu_keyboard())
                .await;
        }

        return Ok(());
    }

    if data == "nav:restart" {
        // Сброс и перезапуск интерфейса в главное меню
        let (_can_read, remaining) = match db.can_make_free_reading(user_id, config.max_free_lifetime_readings).await {
            Ok(res) => res,
            Err(_) => (true, 10),
        };
        let first_name = q.from.first_name.clone();
        let welcome = format!(
            "✨ <b>Бот успешно перезапущен!</b>\n\n\
            Добро пожаловать в «Архитектор реальности», {}!\n\
            Здесь древние знания Таро и звездные матрицы сплетаются с искусственным интеллектом.\n\n\
            🎁 Доступно бесплатных раскладов: <b>{} из {}</b>",
            first_name, remaining, config.max_free_lifetime_readings
        );
        let _ = bot.edit_message_text(chat_id, message_id, welcome)
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
        return Ok(());
    }

    if data == "nav:offer" {
        let bot_name = format!("@{}", config.user_bot_name);
        let text = DETAILED_OFFER_TEXT.replace("@Oraculum_true_bot", &bot_name);
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(legal_menu_keyboard(!db_user.is_offer_accepted))
            .await;
        return Ok(());
    }

    if data == "nav:profile" {
        let history = db.get_user_history(user_id, 10).await.unwrap_or_default();
        let text = format!(
            "👤 <b>Сакральный Профиль Искателя</b>\n\n\
            🆔 <b>ID</b>: <code>{}</code>\n\
            ✨ <b>Имя</b>: {}\n\
            ⚡ <b>Энергия для раскладов</b>: {}\n\
            🔮 <b>Сохраненных раскладов</b>: {}\n\
            📅 <b>В Храме с</b>: {}\n\n\
            <i>Энергия пополняется ежедневно (+1 расклад каждые 24 часа).</i>",
            db_user.telegram_id,
            db_user.first_name,
            db_user.energy_balance,
            history.len(),
            db_user.created_at
        );
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
        return Ok(());
    }

    // 3. Таро: Выбор сферы
    // 10. Меню настроек
    if data == "nav:settings" {
        let text = "⚙️ <b>Настройки Оракула</b>\n\n\
            Управляйте параметрами взаимодействия с искусственным интеллектом и уведомлениями:";
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(settings_keyboard(true, true))
            .await;
        return Ok(());
    }

    if data == "admin:stats" {
        let text = format!("📊 <b>Статистика бота</b>\n\n🗄 База: SQLite (bot.db)\n🔒 Whitelist: <b>@{}</b>", config.allowed_username.as_deref().unwrap_or("Все"));
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback("⬅ Назад", "nav:main")]]))
            .await;
        return Ok(());
    }

    if data == "admin:whitelist" {
        let text = format!("🔒 <b>Whitelist статус</b>\n\nТекущий разрешенный пользователь: <b>@{}</b>\n\n<i>Чтобы изменить пользователя или открыть доступ всем, укажите ALLOWED_USERNAME в .env файле.</i>", config.allowed_username.as_deref().unwrap_or("Все"));
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback("⬅ Назад", "nav:main")]]))
            .await;
        return Ok(());
    }

    if data == "toggle:ai_mode" || data == "toggle:notifications" || data == "toggle:deck_style" {
        let _ = bot.answer_callback_query(query_id).text("✨ Настройка успешно обновлена!").await;
        return Ok(());
    }

    if data == "nav:tarot_spheres" {
        let text = "🃏 <b>Таинство Таро</b>\n\nВ какую сферу вашей жизни направить свет оракула?";
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(tarot_spheres_keyboard())
            .await;
        return Ok(());
    }

    // 4. Таро: Выбор сферы -> выбор подвопроса
    if let Some(sphere_key) = data.strip_prefix("sphere:") {
        let sphere = TAROT_SPHERES.iter().find(|s| s.key == sphere_key);
        let sphere_title = sphere.map(|s| s.title).unwrap_or("Сфера");
        
        let text = format!("📍 <b>{}</b>\n\nУточните глубинный аспект вашего вопроса:", sphere_title);
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(tarot_subtopics_keyboard(sphere_key))
            .await;
        return Ok(());
    }

    // 5. Таро: Выбор подвопроса -> выбор глубины расклада
    if let Some(rest) = data.strip_prefix("subtopic:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() == 2 {
            let sphere_key = parts[0];
            let subtopic_key = parts[1];

            let text = "🃏 <b>Выберите тип и глубину расклада:</b>";
            let _ = bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(tarot_spreads_keyboard(sphere_key, subtopic_key))
                .await;
            return Ok(());
        }
    }

    // 6. Таро: Выбор расклада -> интерактивный выбор карт на столе
    if let Some(rest) = data.strip_prefix("spread:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() == 3 {
            let sphere_key = parts[0];
            let subtopic_key = parts[1];
            let spread_key = parts[2];

            let count = if spread_key == "one_card" { 1 } else { 3 };
            let text = format!(
                "🎴 <b>Колода разложена на сакральном алтаре.</b>\n\n\
                Сделайте вдох, сфокусируйтесь на вопросе и выберите пальцем <b>{}</b> карту(ы):",
                count
            );

            let _ = bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(tarot_pick_cards_keyboard(sphere_key, subtopic_key, spread_key, count, &[]))
                .await;
            return Ok(());
        }
    }

    // 7. Таро: Пошаговый выбор карт и генерация ИИ толкования
    if let Some(rest) = data.strip_prefix("pick:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() == 5 {
            let sphere_key = parts[0];
            let subtopic_key = parts[1];
            let spread_key = parts[2];
            let needed_count: usize = parts[3].parse().unwrap_or(1);
            let picked_raw = parts[4];
            let picked_indices: Vec<usize> = picked_raw
                .split('-')
                .filter_map(|s| s.parse().ok())
                .collect();

            // Если еще не все карты выбраны — обновляем клавиатуру с выбранными
            if picked_indices.len() < needed_count {
                let text = format!(
                    "🎴 Выбрано карт: <b>{}/{}</b>\n\nВыберите следующую карту из оставшихся:",
                    picked_indices.len(),
                    needed_count
                );
                let _ = bot.edit_message_text(chat_id, message_id, text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(tarot_pick_cards_keyboard(sphere_key, subtopic_key, spread_key, needed_count, &picked_indices))
                    .await;
                return Ok(());
            }

            let (can_read, remaining) = match db.can_make_free_reading(user_id, config.max_free_lifetime_readings).await {
                Ok(res) => res,
                Err(_) => (true, 10),
            };

            if !can_read {
                let limit_text = format!(
                    "🔒 <b>Лимит бесплатных раскладов исчерпан</b>\n\n\
                    Вы использовали все <b>{}</b> подарочных раскладов.\n\
                    Для продолжения оформите доступ в разделе <b>Тарифы</b> или обратитесь в поддержку <b>@Studia_taro</b>.",
                    config.max_free_lifetime_readings
                );
                let _ = bot.edit_message_text(chat_id, message_id, limit_text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(tariffs_keyboard())
                    .await;
                return Ok(());
            }

            // ВСЕ КАРТЫ ВЫБРАНЫ: Генерация ИИ
            // Анимация вскрытия
            let loading_text = "🔮 <i>Оракул вскрывает сакральные арканы и соединяется с информационным полем...</i>";
            let _ = bot.edit_message_text(chat_id, message_id, loading_text)
                .parse_mode(ParseMode::Html)
                .await;

            let _ = bot.send_chat_action(chat_id, ChatAction::Typing).await;

            // Вытягиваем случайные карты
            let drawn = TarotDeck::draw_cards(needed_count);
            let sphere = TAROT_SPHERES.iter().find(|s| s.key == sphere_key);
            let sphere_title = sphere.map(|s| s.title).unwrap_or("Сфера");
            let subtopic_title = sphere
                .and_then(|s| s.subtopics.iter().find(|(k, _)| *k == subtopic_key))
                .map(|(_, t)| *t)
                .unwrap_or("Вопрос");
            let spread_info = SPREAD_TYPES.iter().find(|s| s.key == spread_key);
            let spread_name = spread_info.map(|s| s.name).unwrap_or("Расклад");

            let card_tuples: Vec<(&str, bool)> = drawn.iter().map(|(c, rev)| (c.name, *rev)).collect();
            let cards_repr: Vec<String> = drawn.iter().map(|(c, rev)| {
                let pos = if *rev { "Перевернутая" } else { "Прямая" };
                format!("{} ({})", c.name, pos)
            }).collect();

            // Формируем промпт для ИИ
            let user_prompt = build_tarot_prompt(sphere_title, subtopic_title, spread_name, &card_tuples);

            // Запрос к ИИ
            let ai_response = match ai.generate_reading(SYSTEM_ORACLE_PROMPT, &user_prompt).await {
                Ok(res) => res,
                Err(e) => {
                    error!("AI Generation error: {:?}", e);
                    "🔮 <i>В энергетическом поле возникло колебание. Пожалуйста, повторите расклад через несколько минут.</i>".to_string()
                }
            };

            // Сохраняем в историю
            let _ = db.save_reading_history(
                user_id,
                "tarot",
                sphere_title,
                Some(subtopic_title),
                &cards_repr.join(", "),
                &ai_response,
            ).await;

            // Формируем красивый вывод
            let cards_list = cards_repr.iter().map(|c| format!("• <b>{}</b>", c)).collect::<Vec<_>>().join("\n");
            
            let remaining_notice = if remaining > 1 {
                format!("\n🎁 <i>Осталось бесплатных раскладов: {}</i>\n", remaining - 1)
            } else {
                "\n⚠️ <i>Это был ваш последний бесплатный расклад из 10.</i>\n".to_string()
            };

            let final_text = format!(
                "🔮 <b>ТАИНСТВО ТАРО СОВЕРШЕНО</b>\n\n\
                📍 <b>Сфера</b>: {}\n\
                ❓ <b>Вопрос</b>: {}\n\
                🎴 <b>Выпавшие арканы</b>:\n{}\n\
                {}\n\
                ────────────────────\n\
                {}\n\
                ────────────────────",
                sphere_title,
                subtopic_title,
                cards_list,
                remaining_notice,
                ai_response
            );

            let first_card_image = drawn[0].0.image_url;

            // Удаляем старое текстовое сообщение с кнопками выбора
            let _ = bot.delete_message(chat_id, message_id).await;

            // Отправляем полноценное фото первой выпавшей карты с описанием и кнопками
            let _ = bot.send_photo(chat_id, InputFile::url(first_card_image.parse().unwrap()))
                .caption(final_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(contact_tarologist_keyboard(None))
                .await;
            return Ok(());
        }
    }

    // 8. Астрология
    if data == "nav:astrology" {
        let text = "🌌 <b>Астрологический Срез Энергий</b>\n\nВыберите ваш знак Зодиака:";
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(astrology_signs_keyboard())
            .await;
        return Ok(());
    }

    if let Some(sign_key) = data.strip_prefix("astro:") {
        let sign = ZODIAC_SIGNS.iter().find(|z| z.key == sign_key);
        let sign_name = sign.map(|z| z.name).unwrap_or("Зодиак");

        let _ = bot.edit_message_text(chat_id, message_id, "✨ <i>Составляем планетарный срез дня...</i>")
            .parse_mode(ParseMode::Html)
            .await;
        let _ = bot.send_chat_action(chat_id, ChatAction::Typing).await;

        let prompt = build_astrology_prompt(sign_name, "Сегодняшний день");
        let ai_response = match ai.generate_reading(SYSTEM_ORACLE_PROMPT, &prompt).await {
            Ok(res) => res,
            Err(e) => {
                error!("Astrology AI error: {:?}", e);
                "🌌 Не удалось считать планетарный аспект. Попробуйте позже.".to_string()
            }
        };

        // Сохраняем в историю
        let _ = db.save_reading_history(
            user_id,
            "astrology",
            sign_name,
            None,
            sign_name,
            &ai_response,
        ).await;

        let final_text = format!(
            "🌌 <b>АСТРОЛОГИЧЕСКИЙ СРЕЗ: {}</b>\n\n{}\n",
            sign_name, ai_response
        );

        let _ = bot.edit_message_text(chat_id, message_id, final_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
        return Ok(());
    }

    // 9. Игра Лила
    if data == "nav:leela" {
        let (dice, cell) = LeelaGame::roll_and_get_cell();

        let _ = bot.edit_message_text(chat_id, message_id, format!("🎲 <i>Бросаем сакральный кубик... Выпало: <b>{}</b>!</i>", dice))
            .parse_mode(ParseMode::Html)
            .await;
        let _ = bot.send_chat_action(chat_id, ChatAction::Typing).await;

        let prompt = build_leela_prompt(cell.number, cell.name, cell.plane);
        let ai_response = match ai.generate_reading(SYSTEM_ORACLE_PROMPT, &prompt).await {
            Ok(res) => res,
            Err(e) => {
                error!("Leela AI error: {:?}", e);
                "🎲 Клетка Лилы открыта, но толкование временно недоступно.".to_string()
            }
        };

        // Сохраняем в историю
        let _ = db.save_reading_history(
            user_id,
            "leela",
            cell.plane,
            Some(cell.name),
            &format!("Клетка {}", cell.number),
            &ai_response,
        ).await;

        let final_text = format!(
            "🎲 <b>ТРАНСФОРМАЦИОННАЯ ИГРА ЛИЛА</b>\n\n\
            🎯 <b>Бросок кубика</b>: {}\n\
            📍 <b>Клетка №{}</b>: <b>{}</b>\n\
            🌌 <b>План</b>: {}\n\n\
            ────────────────────\n\
            {}\n\
            ────────────────────",
            dice, cell.number, cell.name, cell.plane, ai_response
        );

        let _ = bot.edit_message_text(chat_id, message_id, final_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
        return Ok(());
    }

    Ok(())
}

#[allow(dead_code)]
async fn send_main_menu(bot: &Bot, chat_id: ChatId, first_name: &str, energy: i64) -> ResponseResult<()> {
    let text = format!(
        "🏛 <b>Храм Оракула приветствует вас, {}!</b>\n\n\
        ⚡ Ваш баланс энергии: <b>{}</b>\n\n\
        Выберите таинство для получения ответов:",
        first_name, energy
    );

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(main_menu_keyboard())
        .await?;

    Ok(())
}
