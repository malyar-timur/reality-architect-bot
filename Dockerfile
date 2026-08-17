FROM rust:1.80-slim-bullseye AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
# Кэширование зависимостей
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY . .
RUN touch src/main.rs && cargo build --release

FROM debian:bullseye-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl1.1 sqlite3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/telegram_bot /app/telegram_bot

VOLUME ["/app/data"]
ENV DATABASE_URL="sqlite:///app/data/bot.db?mode=rwc"

CMD ["/app/telegram_bot"]
