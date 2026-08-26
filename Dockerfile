# ====================================================================
# Dockerfile: Fast deployment using pre-built binary
# ====================================================================
FROM debian:bookworm-slim

# Установка необходимых системных библиотек (SSL, SQLite, CA сертификаты)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    sqlite3 \
    tzdata \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем скомпилированный release бинарник и ассеты
COPY telegram_bot /app/telegram_bot
RUN chmod +x /app/telegram_bot
COPY assets /app/assets

# Папка для постоянных данных (SQLite БД и логи)
RUN mkdir -p /app/data
VOLUME ["/app/data"]

# Переменная окружения для базы данных по умолчанию
ENV DATABASE_URL="sqlite:///app/data/bot.db?mode=rwc"
ENV RUST_LOG="info"

CMD ["/app/telegram_bot"]
