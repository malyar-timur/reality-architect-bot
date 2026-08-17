#[allow(dead_code)]
pub struct ZodiacSign {
    pub key: &'static str,
    pub name: &'static str,
    pub element: &'static str,
}

pub const ZODIAC_SIGNS: &[ZodiacSign] = &[
    ZodiacSign { key: "aries", name: "♈ Овен", element: "Огонь" },
    ZodiacSign { key: "taurus", name: "♉ Телец", element: "Земля" },
    ZodiacSign { key: "gemini", name: "♊ Близнецы", element: "Воздух" },
    ZodiacSign { key: "cancer", name: "♋ Рак", element: "Вода" },
    ZodiacSign { key: "leo", name: "♌ Лев", element: "Огонь" },
    ZodiacSign { key: "virgo", name: "♍ Дева", element: "Земля" },
    ZodiacSign { key: "libra", name: "♎ Весы", element: "Воздух" },
    ZodiacSign { key: "scorpio", name: "♏ Скорпион", element: "Вода" },
    ZodiacSign { key: "sagittarius", name: "♐ Стрелец", element: "Огонь" },
    ZodiacSign { key: "capricorn", name: "♑ Козерог", element: "Земля" },
    ZodiacSign { key: "aquarius", name: "♒ Водолей", element: "Воздух" },
    ZodiacSign { key: "pisces", name: "♓ Рыбы", element: "Вода" },
];
