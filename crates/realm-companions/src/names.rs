use rand::Rng;
use crate::companion::Region;

const HOKKAIDO_NAMES: &[&str] = &[
    "Yukina", "Fubuki", "Shiori", "Tsumugi", "Rin", "Koyuki", "Setsu", "Mafuyu",
];

const TOKYO_NAMES: &[&str] = &[
    "Akira", "Mei", "Sora", "Haruka", "Nao", "Riko", "Yui", "Kaede",
];

const OSAKA_NAMES: &[&str] = &[
    "Mako", "Nana", "Kotone", "Hinata", "Chika", "Ayame", "Tamaki", "Ibuki",
];

const KYOTO_NAMES: &[&str] = &[
    "Sakurako", "Sumire", "Miyako", "Tsukasa", "Hotaru", "Shion", "Hisui", "Ran",
];

const HARAJUKU_NAMES: &[&str] = &[
    "Miku", "Rune", "Neon", "Kira", "Luna", "Ema", "Suzu", "Riri",
];

const OKINAWA_NAMES: &[&str] = &[
    "Minami", "Nami", "Coral", "Umi", "Sango", "Asahi", "Hana", "Shiho",
];

const SAPPORO_NAMES: &[&str] = &[
    "Koharu", "Ayaka", "Aoi", "Fuyu", "Misaki", "Chihiro", "Saki", "Kanon",
];

const KANSAI_NAMES: &[&str] = &[
    "Mikoto", "Sayuri", "Wakaba", "Tsubaki", "Yuzuki", "Kasumi", "Momiji", "Akane",
];

pub fn generate(rng: &mut impl Rng, region: &Region) -> String {
    let pool = match region {
        Region::Hokkaido => HOKKAIDO_NAMES,
        Region::Tokyo => TOKYO_NAMES,
        Region::Osaka => OSAKA_NAMES,
        Region::Kyoto => KYOTO_NAMES,
        Region::Harajuku => HARAJUKU_NAMES,
        Region::Okinawa => OKINAWA_NAMES,
        Region::Sapporo => SAPPORO_NAMES,
        Region::Kansai => KANSAI_NAMES,
    };
    pool[rng.random_range(0..pool.len())].to_string()
}
