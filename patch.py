import re

with open("src/handlers.rs", "r", encoding="utf-8") as f:
    code = f.read()

# 1. Исправляем кнопку nav:offer (заменяем edit_message_text на send_message и добавляем динамическое имя)
old_offer = """    if data == "nav:offer" {
        let _ = bot.edit_message_text(chat_id, message_id, DETAILED_OFFER_TEXT)
            .parse_mode(ParseMode::Html)
            .reply_markup(legal_menu_keyboard())
            .await;
        return Ok(());
    }"""
new_offer = """    if data == "nav:offer" {
        let bot_name = format!("@{}", config.user_bot_name);
        let text = DETAILED_OFFER_TEXT.replace("@Oraculum_true_bot", &bot_name);
        let _ = bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(legal_menu_keyboard())
            .await;
        return Ok(());
    }"""
code = code.replace(old_offer, new_offer)

# 2. Исправляем пагинацию оферты (legal:offer:)
old_legal = """            1 => OFFER_PART_1,
            2 => OFFER_PART_2,
            3 => OFFER_PART_3,"""
new_legal = """            1 => OFFER_PART_1,
            2 => OFFER_PART_2,
            3 => OFFER_PART_3,"""
# Надо еще обернуть let text = match ... в replace
old_legal_full = """        let text = match step {
            1 => OFFER_PART_1,
            2 => OFFER_PART_2,
            3 => OFFER_PART_3,
            _ => OFFER_PART_1,
        };

        let _ = bot.edit_message_text(chat_id, message_id, text)"""
new_legal_full = """        let bot_name = format!("@{}", config.user_bot_name);
        let raw_text = match step {
            1 => OFFER_PART_1,
            2 => OFFER_PART_2,
            3 => OFFER_PART_3,
            _ => OFFER_PART_1,
        };
        let text = raw_text.replace("@Oraculum_true_bot", &bot_name);

        let _ = bot.edit_message_text(chat_id, message_id, text)"""
code = code.replace(old_legal_full, new_legal_full)

# 3. Исправляем privacy и consent
code = code.replace(
    "bot.edit_message_text(chat_id, message_id, PRIVACY_POLICY_TEXT)",
    "bot.edit_message_text(chat_id, message_id, &PRIVACY_POLICY_TEXT.replace(\"@Oraculum_true_bot\", &format!(\"@{}\", config.user_bot_name)))"
)
code = code.replace(
    "bot.edit_message_text(chat_id, message_id, CONSENT_TEXT)",
    "bot.edit_message_text(chat_id, message_id, &CONSENT_TEXT.replace(\"@Oraculum_true_bot\", &format!(\"@{}\", config.user_bot_name)))"
)
# Приветственное слово в /start, где DETAILED_OFFER_TEXT (если есть)
code = code.replace(
    "bot.send_message(msg.chat.id, DETAILED_OFFER_TEXT)",
    "bot.send_message(msg.chat.id, &DETAILED_OFFER_TEXT.replace(\"@Oraculum_true_bot\", &format!(\"@{}\", config.user_bot_name)))"
)

with open("src/handlers.rs", "w", encoding="utf-8") as f:
    f.write(code)
print("handlers.rs patched")
