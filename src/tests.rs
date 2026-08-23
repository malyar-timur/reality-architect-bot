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
        
        assert_eq!(user.energy_balance, 0, "По умолчанию дополнительный баланс 0");

        // 1. Проверяем доступ: 1 бесплатный запрос в день
        let status = db.check_access(user.telegram_id, 1).await.expect("Проверка доступа");
        assert!(status.can_access);
        assert_eq!(status.access_type, crate::models::AccessType::DailyFree);
        assert_eq!(status.daily_used_today, 0);

        // 2. Списываем/фиксируем 1-й запрос
        let consumed = db.consume_reading_charge(user.telegram_id, 1).await.expect("Списание заряда");
        assert_eq!(consumed, crate::models::AccessType::DailyFree);

        // 3. Проверяем доступ: бесплатный лимит на сегодня исчерпан, энергии нет -> can_access = false
        let status2 = db.check_access(user.telegram_id, 1).await.expect("Проверка доступа 2");
        assert!(!status2.can_access);
        assert_eq!(status2.daily_used_today, 1);

        // 4. Начисляем +5 дополнительной энергии
        db.add_user_energy(user.telegram_id, 5).await.expect("Начисление энергии");
        let status3 = db.check_access(user.telegram_id, 1).await.expect("Проверка доступа 3");
        assert!(status3.can_access);
        assert_eq!(status3.access_type, crate::models::AccessType::EnergyPackage);
        assert_eq!(status3.energy_balance, 5);

        // 5. Списываем платную энергию
        let consumed2 = db.consume_reading_charge(user.telegram_id, 1).await.expect("Списание энергии");
        assert_eq!(consumed2, crate::models::AccessType::EnergyPackage);
        let user_updated = db.get_user_by_telegram_id(user.telegram_id).await.expect("Получить юзера").unwrap();
        assert_eq!(user_updated.energy_balance, 4);

        // 6. Выдаем Премиум на 30 дней -> безлимит
        db.set_user_premium(user.telegram_id, 30).await.expect("Выдача премиума");
        let status_prem = db.check_access(user.telegram_id, 1).await.expect("Проверка премиум");
        assert!(status_prem.can_access);
        assert_eq!(status_prem.access_type, crate::models::AccessType::Premium);
        assert!(status_prem.is_premium);
    }
}
