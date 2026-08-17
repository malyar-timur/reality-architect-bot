use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use anyhow::Result;
use chrono::Utc;
use crate::models::{DailyLimit, ReadingHistory, User};

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

#[allow(dead_code)]
impl Db {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        // Инициализация структуры базы данных
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                telegram_id INTEGER NOT NULL UNIQUE,
                username TEXT,
                first_name TEXT NOT NULL,
                last_name TEXT,
                is_offer_accepted BOOLEAN NOT NULL DEFAULT 0,
                offer_accepted_at DATETIME,
                energy_balance INTEGER NOT NULL DEFAULT 3,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_active_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS readings_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                reading_type TEXT NOT NULL,
                topic TEXT NOT NULL,
                subtopic TEXT,
                selected_cards TEXT NOT NULL,
                ai_interpretation TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(telegram_id)
            );

            CREATE TABLE IF NOT EXISTS daily_limits (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                count INTEGER NOT NULL DEFAULT 1,
                UNIQUE(user_id, date)
            );
            "#
        )
        .execute(&pool)
        .await?;

        // Авто-миграция для старых баз данных
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN is_offer_accepted BOOLEAN NOT NULL DEFAULT 0;").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN offer_accepted_at DATETIME;").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN energy_balance INTEGER NOT NULL DEFAULT 3;").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN last_active_at DATETIME;").execute(&pool).await;

        Ok(Self { pool })
    }

    /// Регистрация или обновление активности пользователя
    pub async fn create_or_update_user(
        &self,
        telegram_id: i64,
        username: Option<&str>,
        first_name: &str,
        last_name: Option<&str>,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (telegram_id, username, first_name, last_name, last_active_at)
            VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
            ON CONFLICT(telegram_id) DO UPDATE SET
                username = excluded.username,
                first_name = excluded.first_name,
                last_name = excluded.last_name,
                last_active_at = CURRENT_TIMESTAMP
            RETURNING id, telegram_id, username, first_name, last_name, is_offer_accepted, offer_accepted_at, energy_balance, created_at, last_active_at;
            "#
        )
        .bind(telegram_id)
        .bind(username)
        .bind(first_name)
        .bind(last_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Получить пользователя по Telegram ID
    pub async fn get_user_by_telegram_id(&self, telegram_id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, telegram_id, username, first_name, last_name, is_offer_accepted, offer_accepted_at, energy_balance, created_at, last_active_at FROM users WHERE telegram_id = $1"
        )
        .bind(telegram_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Проверить, принял ли пользователь публичную оферту
    pub async fn is_offer_accepted(&self, telegram_id: i64) -> Result<bool> {
        let row: Option<(bool,)> = sqlx::query_as(
            "SELECT is_offer_accepted FROM users WHERE telegram_id = $1"
        )
        .bind(telegram_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0).unwrap_or(false))
    }

    /// Зафиксировать принятие публичной оферты
    pub async fn accept_offer(&self, telegram_id: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users 
            SET is_offer_accepted = 1, 
                offer_accepted_at = CURRENT_TIMESTAMP 
            WHERE telegram_id = $1
            "#
        )
        .bind(telegram_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Проверка и списание энергии или учет ежедневного лимита
    pub async fn can_make_reading(&self, telegram_id: i64, daily_max: i32) -> Result<bool> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        let limit: Option<DailyLimit> = sqlx::query_as(
            "SELECT user_id, date, count FROM daily_limits WHERE user_id = $1 AND date = $2"
        )
        .bind(telegram_id)
        .bind(&today)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(daily) = limit {
            if daily.count < daily_max {
                return Ok(true);
            }
        } else {
            return Ok(true);
        }

        // Если суточный лимит исчерпан, проверяем баланс энергии
        let user = self.get_user_by_telegram_id(telegram_id).await?;
        if let Some(u) = user {
            return Ok(u.energy_balance > 0);
        }

        Ok(false)
    }

    /// Инкремент счетчика раскладов за текущий день
    pub async fn record_reading_usage(&self, telegram_id: i64) -> Result<()> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        sqlx::query(
            r#"
            INSERT INTO daily_limits (user_id, date, count)
            VALUES ($1, $2, 1)
            ON CONFLICT(user_id, date) DO UPDATE SET count = count + 1;
            "#
        )
        .bind(telegram_id)
        .bind(today)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Сохранить результат расклада в историю
    pub async fn save_reading_history(
        &self,
        telegram_id: i64,
        reading_type: &str,
        topic: &str,
        subtopic: Option<&str>,
        selected_cards: &str,
        ai_interpretation: &str,
    ) -> Result<i64> {
        let id = sqlx::query(
            r#"
            INSERT INTO readings_history (user_id, reading_type, topic, subtopic, selected_cards, ai_interpretation)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(telegram_id)
        .bind(reading_type)
        .bind(topic)
        .bind(subtopic)
        .bind(selected_cards)
        .bind(ai_interpretation)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        self.record_reading_usage(telegram_id).await?;

        Ok(id)
    }

    /// Получить историю последних N раскладов пользователя
    pub async fn get_user_history(&self, telegram_id: i64, limit: i64) -> Result<Vec<ReadingHistory>> {
        let history = sqlx::query_as::<_, ReadingHistory>(
            r#"
            SELECT id, user_id, reading_type, topic, subtopic, selected_cards, ai_interpretation, created_at
            FROM readings_history
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#
        )
        .bind(telegram_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }
}
