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
    pub created_at: String,
    pub last_active_at: String,
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
