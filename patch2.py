with open("src/handlers.rs", "r", encoding="utf-8") as f:
    code = f.read()

old_offer = """    if data == "nav:offer" {
        let bot_name = format!("@{}", config.user_bot_name);"""
new_offer = """    if data == "nav:offer" {
        let _ = bot.answer_callback_query(q.id.clone()).await;
        let bot_name = format!("@{}", config.user_bot_name);"""

code = code.replace(old_offer, new_offer)

with open("src/handlers.rs", "w", encoding="utf-8") as f:
    f.write(code)
