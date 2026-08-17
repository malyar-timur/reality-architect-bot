use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TarotCard {
    pub name: &'static str,
    pub arcana_type: &'static str, // "Старший Аркан" или "Младший Аркан"
    pub keywords: &'static str,
    pub image_url: &'static str,
}

pub struct TarotDeck;

impl TarotDeck {
    pub const CARDS: &'static [TarotCard] = &[
        // Старшие Арканы
        TarotCard { name: "0. Шут (Дурак)", arcana_type: "Старший Аркан", keywords: "Начало пути, спонтанность, свобода, открытость новому", image_url: "https://upload.wikimedia.org/wikipedia/commons/9/90/RWS_Tarot_00_Fool.jpg" },
        TarotCard { name: "I. Маг", arcana_type: "Старший Аркан", keywords: "Воля, мастерство, реализация потенциала, активное действие", image_url: "https://upload.wikimedia.org/wikipedia/commons/d/de/RWS_Tarot_01_Magician.jpg" },
        TarotCard { name: "II. Верховная Жрица", arcana_type: "Старший Аркан", keywords: "Интуиция, тайные знания, внутренний голос, пассивность", image_url: "https://upload.wikimedia.org/wikipedia/commons/8/88/RWS_Tarot_02_High_Priestess.jpg" },
        TarotCard { name: "III. Императрица", arcana_type: "Старший Аркан", keywords: "Изобилие, плодородие, созидание, материнская забота", image_url: "https://upload.wikimedia.org/wikipedia/commons/d/d2/RWS_Tarot_03_Empress.jpg" },
        TarotCard { name: "IV. Император", arcana_type: "Старший Аркан", keywords: "Структура, власть, стабильность, авторитет, контроль", image_url: "https://upload.wikimedia.org/wikipedia/commons/c/c3/RWS_Tarot_04_Emperor.jpg" },
        TarotCard { name: "V. Иерофант (Жрец)", arcana_type: "Старший Аркан", keywords: "Традиции, мудрость, духовный наставник, правила", image_url: "https://upload.wikimedia.org/wikipedia/commons/8/8d/RWS_Tarot_05_Hierophant.jpg" },
        TarotCard { name: "VI. Влюбленные", arcana_type: "Старший Аркан", keywords: "Выбор сердца, союз, гармония, искушение", image_url: "https://upload.wikimedia.org/wikipedia/commons/3/3a/TheLovers.jpg" },
        TarotCard { name: "VII. Колесница", arcana_type: "Старший Аркан", keywords: "Прорыв, триумф, контроль над страстями, движение вперед", image_url: "https://upload.wikimedia.org/wikipedia/commons/9/9b/RWS_Tarot_07_Chariot.jpg" },
        TarotCard { name: "VIII. Сила", arcana_type: "Старший Аркан", keywords: "Мягкая сила, уверенность, укрощение внутренних страхов", image_url: "https://upload.wikimedia.org/wikipedia/commons/f/f5/RWS_Tarot_08_Strength.jpg" },
        TarotCard { name: "IX. Отшельник", arcana_type: "Старший Аркан", keywords: "Поиск истины, уединение, самопознание, внутренний свет", image_url: "https://upload.wikimedia.org/wikipedia/commons/4/4d/Tarot_Nine_of_Wands.jpg" },
        TarotCard { name: "X. Колесо Фортуны", arcana_type: "Старший Аркан", keywords: "Цикличность, судьбоносный поворот, удача, перемены", image_url: "https://upload.wikimedia.org/wikipedia/commons/3/3c/RWS_Tarot_10_Wheel_of_Fortune.jpg" },
        TarotCard { name: "XI. Справедливость", arcana_type: "Старший Аркан", keywords: "Карма, баланс, честность, объективное решение", image_url: "https://upload.wikimedia.org/wikipedia/commons/e/e0/RWS_Tarot_11_Justice.jpg" },
        TarotCard { name: "XII. Повешенный", arcana_type: "Старший Аркан", keywords: "Смена перспективы, добровольная пауза, трансформация", image_url: "https://upload.wikimedia.org/wikipedia/commons/2/2b/RWS_Tarot_12_Hanged_Man.jpg" },
        TarotCard { name: "XIII. Смерть", arcana_type: "Старший Аркан", keywords: "Глубокая трансформация, завершение старого, перерождение", image_url: "https://upload.wikimedia.org/wikipedia/commons/d/d7/RWS_Tarot_13_Death.jpg" },
        TarotCard { name: "XIV. Умеренность", arcana_type: "Старший Аркан", keywords: "Баланс, исцеление, терпение, алхимия чувств", image_url: "https://upload.wikimedia.org/wikipedia/commons/f/f8/RWS_Tarot_14_Temperance.jpg" },
        TarotCard { name: "XV. Дьявол", arcana_type: "Старший Аркан", keywords: "Теневые стороны, привязанности, материальные искушения", image_url: "https://upload.wikimedia.org/wikipedia/commons/5/55/RWS_Tarot_15_Devil.jpg" },
        TarotCard { name: "XVI. Башня", arcana_type: "Старший Аркан", keywords: "Крушение иллюзий, внезапный инсайт, освобождение", image_url: "https://upload.wikimedia.org/wikipedia/commons/5/53/RWS_Tarot_16_Tower.jpg" },
        TarotCard { name: "XVII. Звезда", arcana_type: "Старший Аркан", keywords: "Надежда, вдохновение, вера в будущее, путеводный свет", image_url: "https://upload.wikimedia.org/wikipedia/commons/d/db/RWS_Tarot_17_Star.jpg" },
        TarotCard { name: "XVIII. Луна", arcana_type: "Старший Аркан", keywords: "Подсознание, тайны, иллюзии, встреча с неизвестным", image_url: "https://upload.wikimedia.org/wikipedia/commons/7/7f/RWS_Tarot_18_Moon.jpg" },
        TarotCard { name: "XIX. Солнце", arcana_type: "Старший Аркан", keywords: "Ясность, счастье, триумф, витальность, успех", image_url: "https://upload.wikimedia.org/wikipedia/commons/1/17/RWS_Tarot_19_Sun.jpg" },
        TarotCard { name: "XX. Страшный Суд", arcana_type: "Старший Аркан", keywords: "Пробуждение, зов призвания, подведение итогов, прощение", image_url: "https://upload.wikimedia.org/wikipedia/commons/d/dd/RWS_Tarot_20_Judgement.jpg" },
        TarotCard { name: "XXI. Мир", arcana_type: "Старший Аркан", keywords: "Целостность, завершение великого цикла, гармония с миром", image_url: "https://upload.wikimedia.org/wikipedia/commons/f/ff/RWS_Tarot_21_World.jpg" },
        
        // Ключевые Младшие Арканы
        TarotCard { name: "Туз Кубков", arcana_type: "Младший Аркан (Кубки)", keywords: "Чистая любовь, эмоциональный расцвет, интуиция, открытое сердце", image_url: "https://upload.wikimedia.org/wikipedia/commons/3/36/Cups01.jpg" },
        TarotCard { name: "Двойка Кубков", arcana_type: "Младший Аркан (Кубки)", keywords: "Взаимность, родство душ, искренний диалог, гармония", image_url: "https://upload.wikimedia.org/wikipedia/commons/f/f8/Cups02.jpg" },
        TarotCard { name: "Тройка Кубков", arcana_type: "Младший Аркан (Кубки)", keywords: "Празднование, дружба, поддержка сообщества, радость", image_url: "https://upload.wikimedia.org/wikipedia/commons/7/7a/Cups03.jpg" },
        TarotCard { name: "Королева Кубков", arcana_type: "Младший Аркан (Кубки)", keywords: "Чуткость, эмпатия, глубокое понимание, безусловная любовь", image_url: "https://upload.wikimedia.org/wikipedia/commons/6/62/Cups13.jpg" },
        
        TarotCard { name: "Туз Пентаклей", arcana_type: "Младший Аркан (Пентакли)", keywords: "Материальный дар, финансовый шанс, изобилие, фундамент", image_url: "https://upload.wikimedia.org/wikipedia/commons/f/fd/Pents01.jpg" },
        TarotCard { name: "Тройка Пентаклей", arcana_type: "Младший Аркан (Пентакли)", keywords: "Мастерство, признание труда, синергия в работе", image_url: "https://upload.wikimedia.org/wikipedia/commons/4/42/Pents03.jpg" },
        TarotCard { name: "Десятка Пентаклей", arcana_type: "Младший Аркан (Пентакли)", keywords: "Родовое благополучие, устойчивое процветание, наследие", image_url: "https://upload.wikimedia.org/wikipedia/commons/4/42/Pents10.jpg" },
        TarotCard { name: "Король Пентаклей", arcana_type: "Младший Аркан (Пентакли)", keywords: "Финансовая стабильность, надежность, мудрое управление", image_url: "https://upload.wikimedia.org/wikipedia/commons/1/1c/Pents14.jpg" },
        
        TarotCard { name: "Туз Жезлов", arcana_type: "Младший Аркан (Жезлы)", keywords: "Творческая искра, импульс к действию, страсть, вдохновение", image_url: "https://upload.wikimedia.org/wikipedia/commons/1/11/Wands01.jpg" },
        TarotCard { name: "Шестерка Жезлов", arcana_type: "Младший Аркан (Жезлы)", keywords: "Триумф, победа, общественное признание, лидерство", image_url: "https://upload.wikimedia.org/wikipedia/commons/3/3b/Wands06.jpg" },
        TarotCard { name: "Восьмерка Жезлов", arcana_type: "Младший Аркан (Жезлы)", keywords: "Быстрые события, инсайты, движение, новости", image_url: "https://upload.wikimedia.org/wikipedia/commons/6/6b/Wands08.jpg" },
        
        TarotCard { name: "Туз Мечей", arcana_type: "Младший Аркан (Мечи)", keywords: "Ясность мысли, рассечение иллюзий, честность, решение", image_url: "https://upload.wikimedia.org/wikipedia/commons/1/1a/Swords01.jpg" },
        TarotCard { name: "Шестерка Мечей", arcana_type: "Младший Аркан (Мечи)", keywords: "Переход к спокойным водам, путь исцеления, оставление бурь", image_url: "https://upload.wikimedia.org/wikipedia/commons/2/29/Swords06.jpg" },
        TarotCard { name: "Королева Мечей", arcana_type: "Младший Аркан (Мечи)", keywords: "Острый ум, независимость, проницательность, границы", image_url: "https://upload.wikimedia.org/wikipedia/commons/d/d4/Swords13.jpg" },
    ];

    /// Случайное вытягивание N уникальных карт
    pub fn draw_cards(count: usize) -> Vec<(&'static TarotCard, bool)> {
        let mut rng = rand::thread_rng();
        let mut chosen_indices: Vec<usize> = (0..Self::CARDS.len()).collect();
        chosen_indices.shuffle(&mut rng);

        chosen_indices
            .into_iter()
            .take(count)
            .map(|idx| {
                let is_reversed = rng.gen_bool(0.25); // 25% вероятность перевернутой карты
                (&Self::CARDS[idx], is_reversed)
            })
            .collect()
    }
}

pub struct SphereInfo {
    pub key: &'static str,
    pub title: &'static str,
    pub subtopics: &'static [(&'static str, &'static str)], // (subtopic_key, title)
}

pub const TAROT_SPHERES: &[SphereInfo] = &[
    SphereInfo {
        key: "love",
        title: "💖 Любовь и Отношения",
        subtopics: &[
            ("feelings", "Что он/она чувствует ко мне?"),
            ("future", "Перспектива развития союза"),
            ("conflict", "В чем корень недопонимания?"),
            ("advice", "Как гармонизировать отношения?"),
        ],
    },
    SphereInfo {
        key: "career",
        title: "💼 Карьера и Деньги",
        subtopics: &[
            ("money_flow", "Вектор финансового потока"),
            ("job_change", "Стоит ли менять работу / проект?"),
            ("growth", "Точки роста и скрытые возможности"),
            ("blockers", "Что блокирует материальный успех?"),
        ],
    },
    SphereInfo {
        key: "self",
        title: "🧭 Предназначение и Душа",
        subtopics: &[
            ("mission", "В чем мой текущий сакральный урок?"),
            ("talents", "Раскрытие скрытых талантов"),
            ("shadow", "С какой тенью важно встретиться?"),
        ],
    },
    SphereInfo {
        key: "energy",
        title: "⚡ Энергия и Состояние",
        subtopics: &[
            ("balance", "Где теряется жизненный ресурс?"),
            ("source", "Что наполнит силой и вдохновением?"),
            ("day_vibe", "Главная энергия и фокус сегодняшнего дня"),
        ],
    },
];

#[allow(dead_code)]
pub struct SpreadTypeInfo {
    pub key: &'static str,
    pub name: &'static str,
    pub cards_count: usize,
    pub description: &'static str,
}

pub const SPREAD_TYPES: &[SpreadTypeInfo] = &[
    SpreadTypeInfo {
        key: "one_card",
        name: "✨ Карта Дня / Прямой ответ (1 карта)",
        cards_count: 1,
        description: "Быстрый точечный ответ оракула на фокус внимания",
    },
    SpreadTypeInfo {
        key: "three_cards",
        name: "🔮 Триада Времени: Прошлое - Настоящее - Будущее (3 карты)",
        cards_count: 3,
        description: "Глубокий анализ причинно-следственной связи ситуации",
    },
];
