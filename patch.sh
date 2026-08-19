#!/bin/bash
# 1. Замена edit_message_text на send_message в nav:offer и интерполяция имени бота в handlers.rs

# Заменяем nav:offer блок
sed -i 's/let _ = bot.edit_message_text(chat_id, message_id, DETAILED_OFFER_TEXT)/let offer_text = DETAILED_OFFER_TEXT.replace("@Oraculum_true_bot", \&format!("@{}", config.user_bot_name));\n        let _ = bot.send_message(chat_id, offer_text)/g' src/handlers.rs

# Заменяем остальные подстановки DETAILED_OFFER_TEXT, PRIVACY_POLICY_TEXT, CONSENT_TEXT на .replace("@Oraculum_true_bot", &format!("@{}", config.user_bot_name))
# (чтобы было динамически из ENV)
sed -i 's/, DETAILED_OFFER_TEXT/, \&DETAILED_OFFER_TEXT.replace("@Oraculum_true_bot", \&format!("@{}", config.user_bot_name))/g' src/handlers.rs
sed -i 's/bot.send_message(msg.chat.id, DETAILED_OFFER_TEXT)/bot.send_message(msg.chat.id, DETAILED_OFFER_TEXT.replace("@Oraculum_true_bot", \&format!("@{}", config.user_bot_name)))/g' src/handlers.rs

sed -i 's/PRIVACY_POLICY_TEXT)/PRIVACY_POLICY_TEXT.replace("@Oraculum_true_bot", \&format!("@{}", config.user_bot_name)))/g' src/handlers.rs
sed -i 's/CONSENT_TEXT)/CONSENT_TEXT.replace("@Oraculum_true_bot", \&format!("@{}", config.user_bot_name)))/g' src/handlers.rs
sed -i 's/let text = match step {/let bot_name = format!("@{}", config.user_bot_name);\n        let text = match step {/g' src/handlers.rs
sed -i 's/1 => OFFER_PART_1,/1 => \&OFFER_PART_1.replace("@Oraculum_true_bot", \&bot_name),/g' src/handlers.rs
sed -i 's/2 => OFFER_PART_2,/2 => \&OFFER_PART_2.replace("@Oraculum_true_bot", \&bot_name),/g' src/handlers.rs
sed -i 's/3 => OFFER_PART_3,/3 => \&OFFER_PART_3.replace("@Oraculum_true_bot", \&bot_name),/g' src/handlers.rs

# 2. Поправим дефолтный баланс энергии в базе с 10 на 3 (sqlite)
sqlite3 bot.db "UPDATE users SET energy_balance = 3 WHERE energy_balance > 3;"

# 3. Обновляем .env.example
cat << 'ENVA' > .env.example
# ==========================================
# GOLADIA BOT SETTINGS (DevOps Configuration)
# ==========================================

# 1. Telegram Bot Tokens (Токены ботов от @BotFather)
TELOXIDE_TOKEN=ВАШ_ОСНОВНОЙ_ТОКЕН
ADMIN_BOT_TOKEN=ВАШ_ТОКЕН_ДЛЯ_АДМИНКИ

# 2. Bot Usernames (Логины ботов БЕЗ символа @, используются внутри текстов и оферты)
# Пример: Если логин @arch_reality_2026_bot, пишем arch_reality_2026_bot
USER_BOT_NAME=arch_reality_2026_bot
ADMIN_BOT_NAME=arch_settings_bot

# 3. Access Control (Ограничение доступа)
# ADMIN_USERNAMES — кто имеет доступ к боту настроек и админке (через запятую, можно с @ или без)
ADMIN_USERNAMES=mixanik2000,Studia_taro
# ALLOWED_USERNAME — Режим приватного тестирования основного бота. 
# ВАЖНО: Если тут пусто (ALLOWED_USERNAME=), то бот ОТКРЫТ ДЛЯ ВСЕХ ПОЛЬЗОВАТЕЛЕЙ!
ALLOWED_USERNAME=

# 4. Database 
DATABASE_URL=sqlite://bot.db?mode=rwc

# 5. Logging (Ранжирование логов: info, debug, error, warn)
RUST_LOG=info

# 6. AI Provider Configuration (Подключение к нейросети)
AI_BASE_URL=http://.../v1
AI_API_KEY=sk-ваша-нейро-апи-секретка
AI_MODEL=gemini-3.7-flash-high
AI_TIMEOUT_SECS=60

# 7. Business Logic Limits (Настройки бизнес-логики)
# DAILY_FREE_READINGS - Количество бесплатных раскладов (энергии), которые начисляются пользователю КАЖДЫЙ ДЕНЬ
DAILY_FREE_READINGS=3
# MAX_FREE_READINGS - Максимальный жизненный лимит раскладов (пробных) для пользователя, если включена монетизация
MAX_FREE_READINGS=10
ENVA

cp .env.example .env

# Обновляем токены в локальном .env чтобы не сломать текущий стенд
sed -i 's/ВАШ_ОСНОВНОЙ_ТОКЕН/8987639239:AAH82PLjI5aBVeogo_Se9Ef-bYTzIjaFG4o/g' .env
sed -i 's/ВАШ_ТОКЕН_ДЛЯ_АДМИНКИ/8957911866:AAFhlUwT_5B494kJqwT1ljTiAd4JKmeFoOY/g' .env
sed -i 's|http://.../v1|http://192.124.181.128:8045/v1|g' .env
sed -i 's/sk-ваша-нейро-апи-секретка/sk-9565253724374c5db4f0bbec10720f80/g' .env

