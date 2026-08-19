with open("src/handlers.rs", "r", encoding="utf-8") as f:
    code = f.read()

old_block = """    // Если оферта еще не принята — блокируем любые колбэки кроме оферты
    if !db_user.is_offer_accepted {
        let _ = bot.answer_callback_query(q.id).text("Сначала необходимо принять условия оферты").await;
        return Ok(());
    }"""
new_block = """    // Если оферта еще не принята — блокируем любые колбэки кроме оферты
    if !db_user.is_offer_accepted {
        let allowed = data.starts_with("legal:") || data.starts_with("nav:offer") || data.starts_with("nav:legal");
        if !allowed {
            let _ = bot.answer_callback_query(q.id).text("Сначала необходимо принять условия оферты").await;
            return Ok(());
        }
    }"""
code = code.replace(old_block, new_block)

old_start = """        let offer_intro = format!(
            "✨ <b>Добро пожаловать в Сакральный Храм Оракула</b>, {}!\\n\\n{}",
            first_name, DETAILED_OFFER_TEXT
        );"""
new_start = """        let bot_name = format!("@{}", config.user_bot_name);
        let raw_text = DETAILED_OFFER_TEXT.replace("@Oraculum_true_bot", &bot_name);
        let offer_intro = format!(
            "✨ <b>Добро пожаловать в Сакральный Храм Оракула</b>, {}!\\n\\n{}",
            first_name, raw_text
        );"""
code = code.replace(old_start, new_start)

with open("src/handlers.rs", "w", encoding="utf-8") as f:
    f.write(code)
print("Handlers patched")
