import re

with open("src/handlers.rs", "r", encoding="utf-8") as f:
    code = f.read()

# Патчим Карту Дня
old_photo_1 = """        let _ = bot.send_photo(chat_id, InputFile::url(card.image_url.parse().unwrap()))
            .caption(caption)
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await;"""
            
new_photo_1 = """        if let Ok(url) = card.image_url.parse() {
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
        }"""
code = code.replace(old_photo_1, new_photo_1)

with open("src/handlers.rs", "w", encoding="utf-8") as f:
    f.write(code)
print("Anti-crash patch applied")
