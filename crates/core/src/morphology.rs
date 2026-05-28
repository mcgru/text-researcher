//! Morphological features reference data.
//!
//! Data sources:
//! - `docs/morph-features.json` — feature name mappings (Animacy → одуш, etc.)
//! - `docs/table-morph-types.json` — part-of-speech type mappings (NOUN → сущ, etc.)

/// A morphological feature descriptor.
pub struct FeatureInfo {
    pub name_en: &'static str,
    pub short_en: &'static str,
    pub short_ru: &'static str,
    pub medium_ru: &'static str,
}

/// Known morphological features from UD/OpenCorpora.
pub static FEATURES: &[FeatureInfo] = &[
    FeatureInfo { name_en: "Animacy", short_en: "anim", short_ru: "одуш", medium_ru: "одуш" },
    FeatureInfo { name_en: "Aspect", short_en: "asp", short_ru: "вид", medium_ru: "вид" },
    FeatureInfo { name_en: "Case", short_en: "case", short_ru: "пад", medium_ru: "падеж" },
    FeatureInfo { name_en: "Definite", short_en: "def", short_ru: "опр", medium_ru: "опред" },
    FeatureInfo { name_en: "Degree", short_en: "deg", short_ru: "степ", medium_ru: "степень" },
    FeatureInfo { name_en: "Gender", short_en: "gen", short_ru: "род", medium_ru: "род" },
    FeatureInfo { name_en: "Mood", short_en: "mood", short_ru: "накл", medium_ru: "накл" },
    FeatureInfo { name_en: "Number", short_en: "num", short_ru: "чис", medium_ru: "число" },
    FeatureInfo { name_en: "NumType", short_en: "numtype", short_ru: "тип_числ", medium_ru: "тип_числ" },
    FeatureInfo { name_en: "Person", short_en: "pers", short_ru: "лицо", medium_ru: "лицо" },
    FeatureInfo { name_en: "Polarity", short_en: "pol", short_ru: "отриц", medium_ru: "отриц" },
    FeatureInfo { name_en: "Poss", short_en: "poss", short_ru: "прит", medium_ru: "прит" },
    FeatureInfo { name_en: "PronType", short_en: "prontype", short_ru: "тип_мест", medium_ru: "тип_мест" },
    FeatureInfo { name_en: "Reflex", short_en: "reflex", short_ru: "возвр", medium_ru: "возвр" },
    FeatureInfo { name_en: "Tense", short_en: "tense", short_ru: "вр", medium_ru: "время" },
    FeatureInfo { name_en: "VerbForm", short_en: "verbform", short_ru: "форма_гл", medium_ru: "форма_гл" },
    FeatureInfo { name_en: "Voice", short_en: "voice", short_ru: "залог", medium_ru: "залог" },
];

/// A part-of-speech type descriptor.
pub struct PosInfo {
    pub code: &'static str,
    pub short_ru: &'static str,
    pub medium_ru: &'static str,
    pub description_ru: &'static str,
}

/// Known parts of speech from UD/OpenCorpora.
pub static POS_TYPES: &[PosInfo] = &[
    PosInfo { code: "NOUN", short_ru: "с", medium_ru: "сущ", description_ru: "имя существительное" },
    PosInfo { code: "VERB", short_ru: "г", medium_ru: "глаг", description_ru: "личная форма глагола" },
    PosInfo { code: "ADJF", short_ru: "п", medium_ru: "прилп", description_ru: "полное прилагательное" },
    PosInfo { code: "ADJS", short_ru: "к", medium_ru: "прилк", description_ru: "краткое прилагательное" },
    PosInfo { code: "ADVB", short_ru: "н", medium_ru: "нарзн", description_ru: "знаменательное наречие" },
    PosInfo { code: "COMP", short_ru: "ср", medium_ru: "срав", description_ru: "сравнительная степень" },
    PosInfo { code: "PRTF", short_ru: "пч", medium_ru: "причп", description_ru: "полное причастие" },
    PosInfo { code: "PRTS", short_ru: "кч", medium_ru: "причк", description_ru: "краткое причастие" },
    PosInfo { code: "GRND", short_ru: "д", medium_ru: "деепр", description_ru: "деепричастие" },
    PosInfo { code: "INFN", short_ru: "и", medium_ru: "инф", description_ru: "инфинитив" },
    PosInfo { code: "PRED", short_ru: "пд", medium_ru: "пред", description_ru: "предикатив" },
    PosInfo { code: "PREP", short_ru: "пр", medium_ru: "предл", description_ru: "предлог" },
    PosInfo { code: "CONJ", short_ru: "сз", medium_ru: "союз", description_ru: "союз" },
    PosInfo { code: "PRCL", short_ru: "ч", medium_ru: "част", description_ru: "частица" },
    PosInfo { code: "INTJ", short_ru: "м", medium_ru: "межд", description_ru: "междометие" },
    PosInfo { code: "NUMR", short_ru: "чл", medium_ru: "числ", description_ru: "числительное" },
    PosInfo { code: "NPRO", short_ru: "мс", medium_ru: "мест", description_ru: "местоимение" },
    PosInfo { code: "ADV", short_ru: "н", medium_ru: "нареч", description_ru: "наречие" },
];
