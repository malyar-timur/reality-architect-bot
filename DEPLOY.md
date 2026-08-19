# Telegram Bot (Oraculum) Deployment Package

## 📁 Структура директории
- `telegram_bot` — скомпилированный релизный бинарный файл (Rust, x86_64 Linux).
- `Dockerfile` — легковесный образ на базе `debian:bookworm-slim`, использующий готовый бинарник `telegram_bot`.
- `Dockerfile.multistage` — многоэтапный Dockerfile для полной пересборки из исходников (Rust 1.80+).
- `docker-compose.yml` — конфигурация для запуска сервиса с volume для SQLite базы (`./data/bot.db`).
- `.env` / `.env.example` — переменные окружения (токен Telegram бота, API ключи Gemini / OpenAI, параметры БД).
- `src/`, `Cargo.toml`, `Cargo.lock` — исходный код проекта.

## 🚀 Варианты запуска

### Вариант 1: Запуск через Docker Compose (Рекомендуемый)
1. Проверьте и заполните файл `.env`:
   ```bash
   nano .env
   ```
2. Соберите и запустите контейнер в фоне:
   ```bash
   docker compose up -d --build
   ```
3. Проверьте логи:
   ```bash
   docker compose logs -f
   ```

### Вариант 2: Прямой запуск бинарника (без Docker)
1. Убедитесь, что установлены системные библиотеки:
   ```bash
   apt-get update && apt-get install -y ca-certificates libssl3 sqlite3
   ```
2. Запустите бота:
   ```bash
   chmod +x telegram_bot
   ./telegram_bot
   ```

## 🛠️ Управление контейнером
- Остановка: `docker compose down`
- Перезапуск: `docker compose restart`
- Просмотр статуса: `docker compose ps`
- База данных SQLite сохраняется в `./data/bot.db`.
