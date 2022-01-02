use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, LanguageLoader, Localizer,
};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use std::collections::HashMap;

/// Available languages for the language settings `gtk::ComboBoxText`.
/// Builds the `HashMap` of available languages based on directories
/// in the /i18n/ directory.
///
/// The order is sorted later.
pub fn available_languages() -> HashMap<String, String> {
    let mut lang_hashmap: HashMap<String, String> = HashMap::new();

    let available_languages = localizer()
        .language_loader()
        .available_languages(&Localizations);
    if let Ok(languages) = available_languages {
        let languages_dirs = languages
            .iter()
            .map(|l| l.language.to_string())
            .collect::<Vec<String>>();

        let codes = iso6391();
        for lang in languages_dirs {
            lang_hashmap.insert(lang.clone(), codes.get(lang.as_str()).unwrap().to_string());
        }
    } else {
        lang_hashmap.insert("en".to_string(), "English".to_string());
    }

    lang_hashmap
}

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

pub static LANGUAGE_LOADER: Lazy<FluentLanguageLoader> = Lazy::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    loader
});

/// Get the `Localizer` to be used.
pub fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

pub fn iso6391() -> HashMap<&'static str, &'static str> {
    hashmap!(
        "ab" => "Abkhazian",
        "aa" => "Afar",
        "af" => "Afrikaans",
        "ak" => "Akan",
        "sq" => "Albanian",
        "am" => "Amharic",
        "ar" => "Arabic",
        "an" => "Aragonese",
        "hy" => "Armenian",
        "as" => "Assamese",
        "av" => "Avaric",
        "ae" => "Avestan",
        "ay" => "Aymara",
        "az" => "Azerbaijani",
        "bm" => "Bambara",
        "ba" => "Bashkir",
        "eu" => "Basque",
        "be" => "Belarusian",
        "bn" => "Bengali",
        "bh" => "Bihari languages",
        "bi" => "Bislama",
        "bs" => "Bosnian",
        "br" => "Breton",
        "bg" => "Bulgarian",
        "my" => "Burmese",
        "ca" => "Catalan, Valencian",
        "km" => "Central Khmer",
        "ch" => "Chamorro",
        "ce" => "Chechen",
        "ny" => "Chichewa, Chewa, Nyanja",
        "zh" => "Chinese",
        "cu" => "Church Slavonic, Old Bulgarian, Old Church Slavonic",
        "cv" => "Chuvash",
        "kw" => "Cornish",
        "co" => "Corsican",
        "cr" => "Cree",
        "hr" => "Croatian",
        "cs" => "Czech",
        "da" => "Danish",
        "dv" => "Divehi, Dhivehi, Maldivian",
        "nl" => "Dutch, Flemish",
        "dz" => "Dzongkha",
        "en" => "English",
        "eo" => "Esperanto",
        "et" => "Estonian",
        "ee" => "Ewe",
        "fo" => "Faroese",
        "fj" => "Fijian",
        "fi" => "Finnish",
        "fr" => "French",
        "ff" => "Fulah",
        "gd" => "Gaelic, Scottish Gaelic",
        "gl" => "Galician",
        "lg" => "Ganda",
        "ka" => "Georgian",
        "de" => "German",
        "ki" => "Gikuyu, Kikuyu",
        "el" => "Greek (Modern)",
        "kl" => "Greenlandic, Kalaallisut",
        "gn" => "Guarani",
        "gu" => "Gujarati",
        "ht" => "Haitian, Haitian Creole",
        "ha" => "Hausa",
        "he" => "Hebrew",
        "hz" => "Herero",
        "hi" => "Hindi",
        "ho" => "Hiri Motu",
        "hu" => "Hungarian",
        "is" => "Icelandic",
        "io" => "Ido",
        "ig" => "Igbo",
        "id" => "Indonesian",
        "ia" => "Interlingua (International Auxiliary Language Association)",
        "ie" => "Interlingue",
        "iu" => "Inuktitut",
        "ik" => "Inupiaq",
        "ga" => "Irish",
        "it" => "Italian",
        "ja" => "Japanese",
        "jv" => "Javanese",
        "kn" => "Kannada",
        "kr" => "Kanuri",
        "ks" => "Kashmiri",
        "kk" => "Kazakh",
        "rw" => "Kinyarwanda",
        "kv" => "Komi",
        "kg" => "Kongo",
        "ko" => "Korean",
        "kj" => "Kwanyama, Kuanyama",
        "ku" => "Kurdish",
        "ky" => "Kyrgyz",
        "lo" => "Lao",
        "la" => "Latin",
        "lv" => "Latvian",
        "lb" => "Letzeburgesch, Luxembourgish",
        "li" => "Limburgish, Limburgan, Limburger",
        "ln" => "Lingala",
        "lt" => "Lithuanian",
        "lu" => "Luba-Katanga",
        "mk" => "Macedonian",
        "mg" => "Malagasy",
        "ms" => "Malay",
        "ml" => "Malayalam",
        "mt" => "Maltese",
        "gv" => "Manx",
        "mi" => "Maori",
        "mr" => "Marathi",
        "mh" => "Marshallese",
        "ro" => "Moldovan, Moldavian, Romanian",
        "mn" => "Mongolian",
        "na" => "Nauru",
        "nv" => "Navajo, Navaho",
        "nd" => "Northern Ndebele",
        "ng" => "Ndonga",
        "ne" => "Nepali",
        "se" => "Northern Sami",
        "no" => "Norwegian",
        "nb" => "Norwegian Bokmål",
        "nn" => "Norwegian Nynorsk",
        "ii" => "Nuosu, Sichuan Yi",
        "oc" => "Occitan (post 1500)",
        "oj" => "Ojibwa",
        "or" => "Oriya",
        "om" => "Oromo",
        "os" => "Ossetian, Ossetic",
        "pi" => "Pali",
        "pa" => "Panjabi, Punjabi",
        "ps" => "Pashto, Pushto",
        "fa" => "Persian",
        "pl" => "Polish",
        "pt" => "Portuguese",
        "qu" => "Quechua",
        "rm" => "Romansh",
        "rn" => "Rundi",
        "ru" => "Russian",
        "sm" => "Samoan",
        "sg" => "Sango",
        "sa" => "Sanskrit",
        "sc" => "Sardinian",
        "sr" => "Serbian",
        "sn" => "Shona",
        "sd" => "Sindhi",
        "si" => "Sinhala, Sinhalese",
        "sk" => "Slovak",
        "sl" => "Slovenian",
        "so" => "Somali",
        "st" => "Sotho, Southern",
        "nr" => "South Ndebele",
        "es" => "Spanish, Castilian",
        "su" => "Sundanese",
        "sw" => "Swahili",
        "ss" => "Swati",
        "sv" => "Swedish",
        "tl" => "Tagalog",
        "ty" => "Tahitian",
        "tg" => "Tajik",
        "ta" => "Tamil",
        "tt" => "Tatar",
        "te" => "Telugu",
        "th" => "Thai",
        "bo" => "Tibetan",
        "ti" => "Tigrinya",
        "to" => "Tonga (Tonga Islands)",
        "ts" => "Tsonga",
        "tn" => "Tswana",
        "tr" => "Turkish",
        "tk" => "Turkmen",
        "tw" => "Twi",
        "ug" => "Uighur, Uyghur",
        "uk" => "Ukrainian",
        "ur" => "Urdu",
        "uz" => "Uzbek",
        "ve" => "Venda",
        "vi" => "Vietnamese",
        "vo" => "Volap_k",
        "wa" => "Walloon",
        "cy" => "Welsh",
        "fy" => "Western Frisian",
        "wo" => "Wolof",
        "xh" => "Xhosa",
        "yi" => "Yiddish",
        "yo" => "Yoruba",
        "za" => "Zhuang, Chuang",
        "zu" => "Zulu"
    )
}
