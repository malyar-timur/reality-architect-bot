use rand::Rng;

pub struct LeelaCell {
    pub number: u8,
    pub name: &'static str,
    pub plane: &'static str,
}

pub struct LeelaGame;

impl LeelaGame {
    pub const CELLS: &'static [LeelaCell] = &[
        LeelaCell { number: 1, name: "Рождение (Джанма)", plane: "План физического бытия" },
        LeelaCell { number: 2, name: "Иллюзия (Майя)", plane: "План физического бытия" },
        LeelaCell { number: 3, name: "Гнев (Кродха)", plane: "План физического бытия" },
        LeelaCell { number: 4, name: "Жадность (Лобха)", plane: "План физического бытия" },
        LeelaCell { number: 5, name: "Физический план (Бхулока)", plane: "План физического бытия" },
        LeelaCell { number: 6, name: "Заблуждение (Моха)", plane: "План физического бытия" },
        LeelaCell { number: 7, name: "Тщеславие (Мада)", plane: "План физического бытия" },
        LeelaCell { number: 8, name: "Алчность (Матсарья)", plane: "План физического бытия" },
        LeelaCell { number: 9, name: "Чувственный план (Кама-лока)", plane: "План астрального бытия" },
        LeelaCell { number: 10, name: "Очищение (Тапас)", plane: "План астрального бытия" },
        LeelaCell { number: 11, name: "Развлечения (Гандхарвы)", plane: "План астрального бытия" },
        LeelaCell { number: 12, name: "Зависть (Иршья)", plane: "План астрального бытия" },
        LeelaCell { number: 13, name: "Ничтожность (Антарика)", plane: "План астрального бытия" },
        LeelaCell { number: 14, name: "Астральный план (Бхувар-лока)", plane: "План астрального бытия" },
        LeelaCell { number: 15, name: "Фантазия (Нага-лока)", plane: "План астрального бытия" },
        LeelaCell { number: 16, name: "Ревность (Двеша)", plane: "План астрального бытия" },
        LeelaCell { number: 17, name: "Сострадание (Дая)", plane: "План небесного бытия" },
        LeelaCell { number: 18, name: "План радости (Харша-лока)", plane: "План небесного бытия" },
        LeelaCell { number: 19, name: "План действия (Карма-лока)", plane: "План действия" },
        LeelaCell { number: 20, name: "Благотворительность (Дана)", plane: "План действия" },
        LeelaCell { number: 21, name: "Искупление (Прачитта)", plane: "План действия" },
        LeelaCell { number: 22, name: "План Дхармы (Дхарма-лока)", plane: "План действия" },
        LeelaCell { number: 23, name: "Небесный план (Сварга-лока)", plane: "План действия" },
        LeelaCell { number: 24, name: "Плохая компания (Кусанга)", plane: "План действия" },
        LeelaCell { number: 68, name: "Космическое сознание (Вайкунтха-лока)", plane: "План Абсолюта" },
        LeelaCell { number: 72, name: "Слияние с Абсолютом (Тамо-гуна / Мокша)", plane: "План Абсолюта" },
    ];

    /// Бросок кубика (1-6) и случайный выбор клетки
    pub fn roll_and_get_cell() -> (u8, &'static LeelaCell) {
        let mut rng = rand::thread_rng();
        let dice = rng.gen_range(1..=6);
        let cell_idx = rng.gen_range(0..Self::CELLS.len());
        (dice, &Self::CELLS[cell_idx])
    }
}
