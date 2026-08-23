use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub telegram_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub is_offer_accepted: bool,
    pub offer_accepted_at: Option<String>,
    pub energy_balance: i64,
    pub is_premium: bool,
    pub premium_until: Option<String>,
    pub created_at: String,
    pub last_active_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    /// Премиум пользователь — безлимит
    Premium,
    /// Бесплатный ежедневный запрос
    DailyFree,
    /// Использование дополнительного купленного пакета энергии
    EnergyPackage,
}

#[derive(Debug, Clone)]
pub struct UserAccessStatus {
    pub can_access: bool,
    pub access_type: AccessType,
    pub is_premium: bool,
    pub premium_until: Option<String>,
    pub daily_used_today: i32,
    pub daily_limit: i32,
    pub energy_balance: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReadingHistory {
    pub id: i64,
    pub user_id: i64,
    pub reading_type: String,
    pub topic: String,
    pub subtopic: Option<String>,
    pub selected_cards: String,
    pub ai_interpretation: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyLimit {
    pub id: i64,
    pub user_id: i64,
    pub date: String,
    pub count: i32,
}
