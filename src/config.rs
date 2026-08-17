use anyhow::{bail, Context, Result};
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Config {
    pub teloxide_token: String,
    pub database_url: String,
    pub ai_base_url: String,
    pub ai_api_key: String,
    pub ai_model: String,
    pub ai_timeout_secs: u64,
    pub daily_free_readings: i32,
    pub max_free_lifetime_readings: i32,
    pub allowed_username: Option<String>,
    pub admin_usernames: Vec<String>,
    pub user_bot_name: String,
    pub admin_bot_name: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let teloxide_token = env::var("TELOXIDE_TOKEN")
            .context("TELOXIDE_TOKEN must be set in .env or environment")?;

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://bot.db?mode=rwc".to_string());

        let ai_base_url = env::var("AI_BASE_URL")
            .unwrap_or_else(|_| "http://192.124.181.128:8045/v1".to_string());

        let ai_api_key = env::var("AI_API_KEY")
            .context("AI_API_KEY must be set in .env or environment")?;

        let ai_model = env::var("AI_MODEL")
            .unwrap_or_else(|_| "gemini-3.7-flash-high".to_string());

        let ai_timeout_secs = env::var("AI_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);

        let daily_free_readings = env::var("DAILY_FREE_READINGS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        // Лимит 10 бесплатных раскладов на пользователя
        let max_free_lifetime_readings = env::var("MAX_FREE_READINGS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        // Белый список пользователей
        let allowed_username = env::var("ALLOWED_USERNAME").ok();

        // Список администраторов через запятую (например: ADMIN_USERNAMES=mixanik2000,Studia_taro)
        let admin_usernames = env::var("ADMIN_USERNAMES")
            .unwrap_or_else(|_| "mixanik2000,Studia_taro".to_string())
            .split(',')
            .map(|s| s.trim().trim_start_matches('@').to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        // Имена ботов из .env
        let user_bot_name = env::var("USER_BOT_NAME")
            .unwrap_or_else(|_| "arch_reality_2026_bot".to_string());
        let admin_bot_name = env::var("ADMIN_BOT_NAME")
            .unwrap_or_else(|_| "arch_settings_bot".to_string());

        Ok(Self {
            teloxide_token,
            database_url,
            ai_base_url,
            ai_api_key,
            ai_model,
            ai_timeout_secs,
            daily_free_readings,
            max_free_lifetime_readings,
            allowed_username,
            admin_usernames,
            user_bot_name,
            admin_bot_name,
        })
    }

    /// Проверка, является ли пользователь администратором
    pub fn is_admin(&self, username: Option<&str>) -> bool {
        if let Some(name) = username {
            let clean = name.trim_start_matches('@').to_lowercase();
            return self.admin_usernames.iter().any(|adm| adm == &clean);
        }
        false
    }

    /// Инициализация логирования tracing
    pub fn init_logging(&self) {
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "telegram_bot=info,teloxide=info".into());

        let _ = tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .try_init();
    }

    /// Валидация ключевых параметров
    pub fn validate(&self) -> Result<()> {
        if self.teloxide_token.trim().is_empty() {
            bail!("Telegram bot token cannot be empty");
        }
        if self.ai_api_key.trim().is_empty() {
            bail!("AI API key cannot be empty");
        }
        if self.ai_base_url.trim().is_empty() {
            bail!("AI Base URL cannot be empty");
        }
        Ok(())
    }
}
