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
    pub possible_values: &'static [&'static str],
}

/// Known morphological features from UD/OpenCorpora.
pub static FEATURES: &[FeatureInfo] = &[
    FeatureInfo { name_en: "Animacy", short_en: "anim", short_ru: "одуш", medium_ru: "одуш", possible_values: &["Anim", "Inan", "Hum", "Nhum"] },
    FeatureInfo { name_en: "Aspect", short_en: "asp", short_ru: "вид", medium_ru: "вид", possible_values: &["Imp", "Perf", "Hab", "Iter", "Prog", "Prosp"] },
    FeatureInfo { name_en: "Case", short_en: "case", short_ru: "пад", medium_ru: "падеж", possible_values: &["Nom", "Gen", "Dat", "Acc", "Voc", "Loc", "Ins", "Par", "Dis"] },
    FeatureInfo { name_en: "Definite", short_en: "def", short_ru: "опр", medium_ru: "опред", possible_values: &["Ind", "Def", "Spec"] },
    FeatureInfo { name_en: "Degree", short_en: "deg", short_ru: "степ", medium_ru: "степень", possible_values: &["Pos", "Cmp", "Sup", "Equ", "Abs"] },
    FeatureInfo { name_en: "Evidentiality", short_en: "evid", short_ru: "свид", medium_ru: "свид", possible_values: &["Fh", "Nfh", "Direct", "Indirect", "Report", "Infer"] },
    FeatureInfo { name_en: "Foreign", short_en: "foreign", short_ru: "ино", medium_ru: "иностр", possible_values: &["Yes"] },
    FeatureInfo { name_en: "Gender", short_en: "gen", short_ru: "род", medium_ru: "род", possible_values: &["Masc", "Fem", "Neut", "Com"] },
    FeatureInfo { name_en: "Gender[psor]", short_en: "gen_p", short_ru: "род_пос", medium_ru: "род_пос", possible_values: &["Masc", "Fem", "Neut"] },
    FeatureInfo { name_en: "Mood", short_en: "mood", short_ru: "накл", medium_ru: "накл", possible_values: &["Ind", "Imp", "Cnd", "Sub", "Jus", "Pot", "Opt"] },
    FeatureInfo { name_en: "Number", short_en: "num", short_ru: "чис", medium_ru: "число", possible_values: &["Sing", "Plur", "Dual", "Ptan", "Coll"] },
    FeatureInfo { name_en: "Number[psor]", short_en: "num_p", short_ru: "чис_пос", medium_ru: "число_пос", possible_values: &["Sing", "Plur"] },
    FeatureInfo { name_en: "NumType", short_en: "numtype", short_ru: "тип_числ", medium_ru: "тип_числ", possible_values: &["Card", "Ord", "Mult", "Frac", "Sets", "Dist", "Range"] },
    FeatureInfo { name_en: "Person", short_en: "pers", short_ru: "лицо", medium_ru: "лицо", possible_values: &["1", "2", "3"] },
    FeatureInfo { name_en: "Polarity", short_en: "pol", short_ru: "отриц", medium_ru: "отриц", possible_values: &["Neg", "Pos"] },
    FeatureInfo { name_en: "Politeness", short_en: "polite", short_ru: "вежл", medium_ru: "вежл", possible_values: &["Infm", "Form", "Elev", "Humb"] },
    FeatureInfo { name_en: "Poss", short_en: "poss", short_ru: "прит", medium_ru: "прит", possible_values: &["Yes"] },
    FeatureInfo { name_en: "PronType", short_en: "prontype", short_ru: "тип_мест", medium_ru: "тип_мест", possible_values: &["Prs", "Rcp", "Int", "Rel", "Dem", "Emp", "Tot", "Ind", "Neg"] },
    FeatureInfo { name_en: "Reflex", short_en: "reflex", short_ru: "возвр", medium_ru: "возвр", possible_values: &["Yes"] },
    FeatureInfo { name_en: "Tense", short_en: "tense", short_ru: "вр", medium_ru: "время", possible_values: &["Past", "Pres", "Fut", "Pqp", "Imp"] },
    FeatureInfo { name_en: "VerbForm", short_en: "verbform", short_ru: "форма_гл", medium_ru: "форма_гл", possible_values: &["Fin", "Inf", "Part", "Conv", "Ger", "Vnoun", "Sup"] },
    FeatureInfo { name_en: "Voice", short_en: "voice", short_ru: "залог", medium_ru: "залог", possible_values: &["Act", "Pass", "Mid", "Cau"] },
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

/// Build a compact feature string in format: `@lem:word, @pos:NOUN, @case:Nom, @gen:Masc, ...`
///
/// Takes grammeme codes (OpenCorpora values like "sing", "nomn", "masc") and
/// maps them to short feature names ("num", "case", "gen").
pub fn compact_features(lemma: &str, pos: &str, grammeme_codes: &[String]) -> String {
    let mut parts = vec![format!("@lem:{}", lemma)];
    if !pos.is_empty() {
        parts.push(format!("@pos:{}", pos));
    }
    for code in grammeme_codes {
        // Map grammeme code to feature short name
        // e.g., "sing" → look up in FEATURES[Number].possible_values → "num"
        let feature_name = feature_for_value(code);
        parts.push(format!("@{}:{}", feature_name, code));
    }
    parts.join(", ")
}

/// Find the feature short name for a grammeme value (e.g., "sing" → "num").
fn feature_for_value(value: &str) -> &str {
    for feat in FEATURES {
        for &pv in feat.possible_values {
            if pv.to_lowercase() == value.to_lowercase() {
                return feat.short_en;
            }
        }
    }
    // Fallback: use the value itself as the feature name if not found
    value
}

/// Look up the short English code for a feature name (e.g., "Gender" → "gen").
pub fn feature_short(name_en: &str) -> Option<&str> {
    FEATURES.iter().find(|f| f.name_en == name_en).map(|f| f.short_en)
}

