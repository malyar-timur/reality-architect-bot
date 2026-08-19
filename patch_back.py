with open("src/handlers.rs", "r", encoding="utf-8") as f:
    code = f.read()

# Разрешаем nav:main_menu в блоке для непринятой оферты
old_allowed = 'let allowed = data.starts_with("legal:") || data.starts_with("nav:offer") || data.starts_with("nav:legal");'
new_allowed = 'let allowed = data.starts_with("legal:") || data.starts_with("nav:offer") || data.starts_with("nav:legal") || data == "nav:main_menu";'
code = code.replace(old_allowed, new_allowed)

# Изменяем nav:main_menu чтобы он возвращал на экран приветствия, если оферта не принята
old_nav_menu = """    if data == "nav:main_menu" {
        let text = format!(
            "🏛 <b>Главный зал Оракула</b>\\n\\n\\
            Ваш запас энергии: ⚡ <b>{}</b>\\n\\
            Выберите желаемое таинство:",
            db_user.energy_balance
        );
        let _ = bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;
        return Ok(());
    }"""

new_nav_menu = """    if data == "nav:main_menu" {
        let _ = bot.answer_callback_query(q.id.clone()).await;
        
        if !db_user.is_offer_accepted {
            let bot_name = format!("@{}", config.user_bot_name);
            let raw_text = DETAILED_OFFER_TEXT.replace("@Oraculum_true_bot", &bot_name);
            let offer_intro = format!(
                "✨ <b>Добро пожаловать в Сакральный Храм Оракула</b>, {}!\\n\\n{}",
                db_user.first_name, raw_text
            );
            let _ = bot.send_message(chat_id, offer_intro)
                .parse_mode(ParseMode::Html)
                .reply_markup(offer_keyboard())
                .await;
            return Ok(());
        }

        let text = format!(
            "🏛 <b>Главный зал Оракула</b>\\n\\n\\
            Ваш запас энергии: ⚡ <b>{}</b>\\n\\
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
    }"""
code = code.replace(old_nav_menu, new_nav_menu)

with open("src/handlers.rs", "w", encoding="utf-8") as f:
    f.write(code)
print("Nav menu patched")
