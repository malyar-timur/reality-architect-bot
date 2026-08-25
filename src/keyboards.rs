use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::esoterics::{TAROT_SPHERES, SPREAD_TYPES};
use crate::esoterics::astrology::ZODIAC_SIGNS;

/// Клавиатура принятия публичной оферты
pub fn offer_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("✅ Я принимаю условия оферты", "accept_offer"),
        ],
        vec![
            InlineKeyboardButton::callback("📜 Подробнее об условиях", "nav:offer"),
        ],
    ])
}

/// Главное меню ORACULUM
pub fn main_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🔮 Сделать расклад Таро", "nav:tarot_spheres"),
        ],
        vec![
            InlineKeyboardButton::callback("🌙 Оракул: Карта дня", "nav:card_of_the_day"),
        ],
        vec![
            InlineKeyboardButton::callback("🌌 Астрология", "nav:astrology"),
            InlineKeyboardButton::callback("🎲 Игра Лила", "nav:leela"),
        ],
        vec![
            InlineKeyboardButton::callback("💎 Тарифы", "nav:tariffs"),
            InlineKeyboardButton::callback("📚 История", "nav:history"),
        ],
        vec![
            InlineKeyboardButton::callback("👤 Профиль", "nav:profile"),
            InlineKeyboardButton::callback("🛟 Поддержка", "nav:support"),
        ],
        vec![
            InlineKeyboardButton::callback("🔄 Перезапустить бота", "nav:restart"),
        ],
    ])
}

/// Выбор сферы Таро
pub fn tarot_spheres_keyboard() -> InlineKeyboardMarkup {
    let mut rows = Vec::new();
    for sphere in TAROT_SPHERES {
        rows.push(vec![
            InlineKeyboardButton::callback(sphere.title, format!("sphere:{}", sphere.key)),
        ]);
    }
    rows.push(vec![
        InlineKeyboardButton::callback("🏠 Главное меню", "nav:main_menu"),
    ]);
    InlineKeyboardMarkup::new(rows)
}

/// Выбор подвопроса внутри сферы Таро
pub fn tarot_subtopics_keyboard(sphere_key: &str) -> InlineKeyboardMarkup {
    let sphere = TAROT_SPHERES.iter().find(|s| s.key == sphere_key);
    let mut rows = Vec::new();

    if let Some(s) = sphere {
        for (sub_key, sub_title) in s.subtopics {
            rows.push(vec![
                InlineKeyboardButton::callback(*sub_title, format!("subtopic:{}:{}", sphere_key, sub_key)),
            ]);
        }
    }

    rows.push(vec![
        InlineKeyboardButton::callback("◀️ Назад к сферам", "nav:tarot_spheres"),
        InlineKeyboardButton::callback("🏠 Меню", "nav:main_menu"),
    ]);
    InlineKeyboardMarkup::new(rows)
}

/// Выбор типа расклада Таро
pub fn tarot_spreads_keyboard(sphere_key: &str, subtopic_key: &str) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();
    for spread in SPREAD_TYPES {
        rows.push(vec![
            InlineKeyboardButton::callback(
                spread.name,
                format!("spread:{}:{}:{}", sphere_key, subtopic_key, spread.key),
            ),
        ]);
    }
    rows.push(vec![
        InlineKeyboardButton::callback("◀️ Назад", format!("sphere:{}", sphere_key)),
        InlineKeyboardButton::callback("🏠 Меню", "nav:main_menu"),
    ]);
    InlineKeyboardMarkup::new(rows)
}

/// Интерактивный выбор закрытых карт (колода на столе)
pub fn tarot_pick_cards_keyboard(
    sphere_key: &str,
    subtopic_key: &str,
    spread_key: &str,
    needed_count: usize,
    picked_indices: &[usize],
) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();
    
    // 6 карт колоды с сакральным символом рубашки Таро 🎴
    let card_labels = [
        "🎴 Карточка", "🎴 Карточка", "🎴 Карточка",
        "🎴 Карточка", "🎴 Карточка", "🎴 Карточка"
    ];
    let mut row1 = Vec::new();
    let mut row2 = Vec::new();

    for i in 0..6 {
        let is_picked = picked_indices.contains(&i);
        let label = if is_picked { "✨ Открыта" } else { card_labels[i] };
        let callback = if is_picked {
            "noop".to_string()
        } else {
            let mut new_picked = picked_indices.to_vec();
            new_picked.push(i);
            let picked_str: Vec<String> = new_picked.iter().map(|n| n.to_string()).collect();
            format!("pick:{}:{}:{}:{}:{}", sphere_key, subtopic_key, spread_key, needed_count, picked_str.join("-"))
        };

        let btn = InlineKeyboardButton::callback(label, callback);
        if i < 3 {
            row1.push(btn);
        } else {
            row2.push(btn);
        }
    }

    rows.push(row1);
    rows.push(row2);
    rows.push(vec![
        InlineKeyboardButton::callback("◀️ Назад", format!("sphere:{}", sphere_key)),
        InlineKeyboardButton::callback("🏠 Меню", "nav:main_menu"),
    ]);

    InlineKeyboardMarkup::new(rows)
}

/// Клавиатура настроек бота (Settings)
pub fn settings_keyboard(is_ai_enhanced: bool, notifications_enabled: bool) -> InlineKeyboardMarkup {
    let ai_text = if is_ai_enhanced { "🧠 ИИ-Анализ: Вкл ✅" } else { "🧠 ИИ-Анализ: Выкл ❌" };
    let notif_text = if notifications_enabled { "🔔 Утренний Оракул: Вкл ✅" } else { "🔔 Утренний Оракул: Выкл ❌" };

    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(ai_text, "toggle:ai_mode"),
        ],
        vec![
            InlineKeyboardButton::callback(notif_text, "toggle:notifications"),
        ],
        vec![
            InlineKeyboardButton::callback("🎴 Стиль колоды: Классика Уэйта", "toggle:deck_style"),
        ],
        vec![
            InlineKeyboardButton::callback("🏠 Главное меню", "nav:main_menu"),
        ],
    ])
}

/// Выбор знака Зодиака
pub fn astrology_signs_keyboard() -> InlineKeyboardMarkup {
    let mut rows = Vec::new();
    let chunks = ZODIAC_SIGNS.chunks(2);

    for chunk in chunks {
        let row = chunk
            .iter()
            .map(|z| InlineKeyboardButton::callback(z.name, format!("astro:{}", z.key)))
            .collect();
        rows.push(row);
    }

    rows.push(vec![
        InlineKeyboardButton::callback("🏠 Главное меню", "nav:main_menu"),
    ]);
    InlineKeyboardMarkup::new(rows)
}

/// Клавиатура экрана "Связаться с тарологом"
pub fn contact_tarologist_keyboard(payment_url: Option<&str>) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();
    if let Some(url) = payment_url {
        rows.push(vec![
            InlineKeyboardButton::url("💳 Оплатить 100 ₽ (ЮKassa / ЮMoney)", url.parse().unwrap()),
        ]);
    } else {
        rows.push(vec![
            InlineKeyboardButton::callback("💳 Оплатить 100 ₽", "pay:tarologist_consultation"),
        ]);
    }
    rows.push(vec![
        InlineKeyboardButton::callback("🔮 Сделать ещё расклад", "nav:tarot_spheres"),
        InlineKeyboardButton::callback("🏠 Главное меню", "nav:main_menu"),
    ]);
    InlineKeyboardMarkup::new(rows)
}

/// Клавиатура экрана "Тарифы и подписка"
pub fn tariffs_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("⚡ Пакет «5 Раскладов» — 290 ₽", "pay:pack_5"),
        ],
        vec![
            InlineKeyboardButton::callback("🌟 Безлимит «Адепт» (месяц) — 790 ₽", "pay:sub_month"),
        ],
        vec![
            InlineKeyboardButton::callback("💫 Безлимит «Маг» (3 месяца) — 1990 ₽", "pay:sub_3months"),
        ],
        vec![
            InlineKeyboardButton::callback("🔮 Безлимит «Верховная Жрица» (год) — 5990 ₽", "pay:sub_year"),
        ],
        vec![
            InlineKeyboardButton::callback("← Главное меню", "nav:main_menu"),
        ],
    ])
}

/// Клавиатура экрана "Поддержка"
pub fn support_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::url("💬 Написать в поддержку", "https://t.me/Studia_taro".parse().unwrap()),
        ],
        vec![
            InlineKeyboardButton::callback("← Главное меню", "nav:main_menu"),
        ],
    ])
}
