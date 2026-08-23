use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use crate::models::{AccessType, ReadingHistory, User, UserAccessStatus};

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
                energy_balance INTEGER NOT NULL DEFAULT 0,
                is_premium BOOLEAN NOT NULL DEFAULT 0,
                premium_until DATETIME,
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

        // Авто-миграция для существующих баз данных
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN is_offer_accepted BOOLEAN NOT NULL DEFAULT 0;").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN offer_accepted_at DATETIME;").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN energy_balance INTEGER NOT NULL DEFAULT 0;").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN is_premium BOOLEAN NOT NULL DEFAULT 0;").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE users ADD COLUMN premium_until DATETIME;").execute(&pool).await;
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
            RETURNING id, telegram_id, username, first_name, last_name, is_offer_accepted, offer_accepted_at, energy_balance, is_premium, premium_until, created_at, last_active_at;
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

    /// Установить точное количество раскладов всем пользователям
    pub async fn set_all_users_spreads(&self, count: i64) -> Result<()> {
        sqlx::query("UPDATE users SET energy_balance = $1")
            .bind(count)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Добавить пакет раскладов конкретному пользователю
    pub async fn add_user_energy(&self, telegram_id: i64, count: i64) -> Result<()> {
        sqlx::query("UPDATE users SET energy_balance = energy_balance + $1 WHERE telegram_id = $2")
            .bind(count)
            .bind(telegram_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Выдать премиум статус на N дней (если уже есть премиум — продлевает)
    pub async fn set_user_premium(&self, telegram_id: i64, days: i64) -> Result<String> {
        self.grant_premium_days(telegram_id, days).await
    }

    /// Выдать премиум статус на N дней (если уже есть премиум — продлевает)
    pub async fn grant_premium_days(&self, telegram_id: i64, days: i64) -> Result<String> {
        let user = self.get_user_by_telegram_id(telegram_id).await?;
        let now = Utc::now();

        let base_time = match user {
            Some(u) => {
                if let Some(until_str) = u.premium_until {
                    if let Ok(parsed) = DateTime::parse_from_rfc3339(&until_str) {
                        let parsed_utc = parsed.with_timezone(&Utc);
                        if parsed_utc > now {
                            parsed_utc
                        } else {
                            now
                        }
                    } else {
                        now
                    }
                } else {
                    now
                }
            }
            None => now,
        };

        let new_until = base_time + Duration::days(days);
        let new_until_str = new_until.to_rfc3339();

        sqlx::query(
            "UPDATE users SET is_premium = 1, premium_until = $1 WHERE telegram_id = $2"
        )
        .bind(&new_until_str)
        .bind(telegram_id)
        .execute(&self.pool)
        .await?;

        Ok(new_until_str)
    }

    /// Аннулировать премиум статус
    pub async fn revoke_premium(&self, telegram_id: i64) -> Result<()> {
        sqlx::query("UPDATE users SET is_premium = 0, premium_until = NULL WHERE telegram_id = $1")
            .bind(telegram_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Получить всех пользователей для админки
    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    /// Получить пользователя по Telegram ID
    pub async fn get_user_by_telegram_id(&self, telegram_id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, telegram_id, username, first_name, last_name, is_offer_accepted, offer_accepted_at, energy_balance, is_premium, premium_until, created_at, last_active_at FROM users WHERE telegram_id = $1"
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

    /// Проверить статус доступа к ИИ (Премиум / 1 бесплатный в день / купленная энергия)
    pub async fn check_access(&self, telegram_id: i64, daily_limit: i32) -> Result<UserAccessStatus> {
        let user = self.get_user_by_telegram_id(telegram_id).await?;
        let today = Utc::now().format("%Y-%m-%d").to_string();

        let count: i32 = sqlx::query_scalar(
            "SELECT COALESCE(count, 0) FROM daily_limits WHERE user_id = $1 AND date = $2"
        )
        .bind(telegram_id)
        .bind(&today)
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or(0);

        let (is_active_premium, premium_until_str, energy_balance) = match &user {
            Some(u) => {
                let is_prem = if u.is_premium {
                    if let Some(ref until_str) = u.premium_until {
                        if let Ok(parsed) = DateTime::parse_from_rfc3339(until_str) {
                            parsed.with_timezone(&Utc) > Utc::now()
                        } else {
                            false
                        }
                    } else {
                        true
                    }
                } else {
                    false
                };
                (is_prem, u.premium_until.clone(), u.energy_balance)
            }
            None => (false, None, 0),
        };

        if is_active_premium {
            return Ok(UserAccessStatus {
                can_access: true,
                access_type: AccessType::Premium,
                is_premium: true,
                premium_until: premium_until_str,
                daily_used_today: count,
                daily_limit,
                energy_balance,
            });
        }

        // Если есть бесплатный запрос на сегодня
        if count < daily_limit {
            return Ok(UserAccessStatus {
                can_access: true,
                access_type: AccessType::DailyFree,
                is_premium: false,
                premium_until: None,
                daily_used_today: count,
                daily_limit,
                energy_balance,
            });
        }

        // Если исчерпан суточный лимит, проверяем платный баланс энергии
        if energy_balance > 0 {
            return Ok(UserAccessStatus {
                can_access: true,
                access_type: AccessType::EnergyPackage,
                is_premium: false,
                premium_until: None,
                daily_used_today: count,
                daily_limit,
                energy_balance,
            });
        }

        // Лимит исчерпан
        Ok(UserAccessStatus {
            can_access: false,
            access_type: AccessType::DailyFree,
            is_premium: false,
            premium_until: None,
            daily_used_today: count,
            daily_limit,
            energy_balance,
        })
    }

    /// Списать один запрос к ИИ (с учетом премиума, дневного лимита или баланса энергии)
    pub async fn consume_reading_charge(&self, telegram_id: i64, daily_limit: i32) -> Result<AccessType> {
        let status = self.check_access(telegram_id, daily_limit).await?;
        let today = Utc::now().format("%Y-%m-%d").to_string();

        match status.access_type {
            AccessType::Premium => {
                // Премиум не списывает баланс, но записывает счетчик для статистики
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

                Ok(AccessType::Premium)
            }
            AccessType::DailyFree => {
                // Использован бесплатный суточный запрос
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

                Ok(AccessType::DailyFree)
            }
            AccessType::EnergyPackage => {
                // Списываем 1 расклад из купленного пакета
                sqlx::query(
                    "UPDATE users SET energy_balance = MAX(0, energy_balance - 1) WHERE telegram_id = $1"
                )
                .bind(telegram_id)
                .execute(&self.pool)
                .await?;

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

                Ok(AccessType::EnergyPackage)
            }
        }
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
