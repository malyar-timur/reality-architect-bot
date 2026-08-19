#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::db::Db;
    use crate::esoterics::tarot::TarotDeck;
    use crate::esoterics::astrology::ZODIAC_SIGNS;
    use crate::esoterics::leela::LeelaGame;

    #[test]
    fn test_tarot_deck_cards() {
        assert_eq!(TarotDeck::CARDS.len(), 36, "Все карты должны присутствовать в колоде");
        let drawn = TarotDeck::draw_cards(3);
        assert_eq!(drawn.len(), 3, "Должно быть вытянуто ровно 3 карты");
    }

    #[test]
    fn test_astrology_zodiac_signs() {
        assert_eq!(ZODIAC_SIGNS.len(), 12, "Должно быть ровно 12 знаков Зодиака");
        let aries = ZODIAC_SIGNS.iter().find(|s| s.key == "aries");
        assert!(aries.is_some(), "Овен должен находиться в базе знаков");
    }

    #[test]
    fn test_leela_game_cells() {
        assert!(!LeelaGame::CELLS.is_empty(), "Клетки Лилы должны быть инициализированы");
        let cell_1 = &LeelaGame::CELLS[0];
        assert_eq!(cell_1.number, 1);
    }

    #[test]
    fn test_admin_whitelist_check() {
        let mut config = Config {
            teloxide_token: "test".to_string(),
            database_url: "sqlite::memory:".to_string(),
            ai_base_url: "http://localhost".to_string(),
            ai_api_key: "key".to_string(),
            ai_model: "model".to_string(),
            ai_timeout_secs: 30,
            daily_free_readings: 3,
            max_free_lifetime_readings: 10,
            allowed_username: Some("Studia_taro".to_string()),
            admin_usernames: vec!["mixanik2000".to_string(), "Studia_taro".to_string()],
            user_bot_name: "bot".to_string(),
            admin_bot_name: "admin".to_string(),
        };
        config.admin_usernames = vec!["mixanik2000".to_string(), "Studia_taro".to_string()];
        
        assert!(config.is_admin(Some("mixanik2000")), "mixanik2000 должен иметь доступ к админке");
        assert!(config.is_admin(Some("@mixanik2000")), "@mixanik2000 с собачкой должен иметь доступ к админке");
        assert!(config.is_admin(Some("Studia_taro")), "Studia_taro должна иметь доступ к админке");
        assert!(!config.is_admin(Some("hacker_1337")), "Посторонний не должен иметь доступ к админке");
    }

    #[tokio::test]
    async fn test_in_memory_database_user_spreads() {
        let db = Db::new("sqlite::memory:").await.expect("База SQLite в памяти должна инициализироваться");
        
        let user = db.create_or_update_user(123456789, Some("Studia_taro"), "Studia", None)
            .await
            .expect("Пользователь должен создаваться");
        
        assert_eq!(user.energy_balance, 10, "По умолчанию должно начисляться 10 бесплатных раскладов");

        let (can_read, remaining) = db.can_make_free_reading(user.telegram_id, 10).await.expect("Проверка баланса");
        assert!(can_read);
        assert_eq!(remaining, 10);

        db.save_reading_history(user.telegram_id, "tarot", "one_card", None, "Шут", "Толкование").await.expect("Запись расклада в историю");
        let (can_read_after, remaining_after) = db.can_make_free_reading(user.telegram_id, 10).await.expect("Проверка после списания");
        assert!(can_read_after);
        assert_eq!(remaining_after, 9, "Остаток должен уменьшиться до 9");

        db.set_all_users_spreads(25).await.expect("Массовое изменение баланса из админки");
    }
}
