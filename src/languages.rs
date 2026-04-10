pub const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("de", "Deutsch"),
    ("fr", "Français"),
    ("it", "Italiano"),
    ("es", "Español"),
    ("pt", "Português"),
    ("ja", "日本語"),
    ("ko", "한국어"),
    ("zh", "中文"),
    ("nl", "Nederlands"),
    ("sv", "Svenska"),
    ("da", "Dansk"),
    ("fi", "Suomi"),
    ("nb", "Norsk"),
    ("pl", "Polski"),
    ("ru", "Русский"),
    ("tr", "Türkçe"),
    ("ar", "العربية"),
    ("th", "ไทย"),
    ("vi", "Tiếng Việt"),
];

pub fn language_name(code: &str) -> &str {
    LANGUAGES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, name)| *name)
        .unwrap_or(code)
}
