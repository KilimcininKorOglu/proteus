use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum LocaleCategory {
    LcAll,
    LcCtype,
    Lang,
}

#[derive(Debug, Clone)]
pub struct Locale {
    pub lang: String,
    pub encoding: String,
    pub utf8: bool,
}

impl Locale {
    pub fn detect() -> Self {
        let lc_all = env::var("LC_ALL").ok();
        let lc_ctype = env::var("LC_CTYPE").ok();
        let lang = env::var("LANG").ok();

        let locale_str = lc_all
            .or(lc_ctype)
            .or(lang)
            .unwrap_or_else(|| "C".to_string());

        let utf8 = locale_str.to_uppercase().contains("UTF-8")
            || locale_str.to_uppercase().contains("UTF8");

        let (lang, encoding) = if let Some(dot) = locale_str.find('.') {
            (locale_str[..dot].to_string(), locale_str[dot + 1..].to_string())
        } else {
            (locale_str.clone(), "ASCII".to_string())
        };

        Locale { lang, encoding, utf8 }
    }

    pub fn is_turkish(&self) -> bool {
        self.lang.starts_with("tr") || self.lang.starts_with("TR")
    }
}
