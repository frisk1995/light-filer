use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum FontChoice {
    Meiryo,
    YuGothic,
    BizUdGothic,
    MsGothic,
    NotoSansJp,
    IbmPlexSans,
}

impl FontChoice {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Meiryo      => "メイリオ",
            Self::YuGothic    => "Yu Gothic",
            Self::BizUdGothic => "BIZ UD Gothic",
            Self::MsGothic    => "MS Gothic",
            Self::NotoSansJp  => "Noto Sans JP (内蔵)",
            Self::IbmPlexSans => "IBM Plex Sans (内蔵)",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Meiryo      => "meiryo",
            Self::YuGothic    => "yugothic",
            Self::BizUdGothic => "bizudgothic",
            Self::MsGothic    => "msgothic",
            Self::NotoSansJp  => "notosansjp",
            Self::IbmPlexSans => "ibmplexsans",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "yugothic"    => Self::YuGothic,
            "bizudgothic" => Self::BizUdGothic,
            "msgothic"    => Self::MsGothic,
            "notosansjp"  => Self::NotoSansJp,
            "ibmplexsans" => Self::IbmPlexSans,
            _             => Self::Meiryo,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Meiryo,
            Self::YuGothic,
            Self::BizUdGothic,
            Self::MsGothic,
            Self::NotoSansJp,
            Self::IbmPlexSans,
        ]
    }
}

pub struct Saved {
    pub theme_dark: bool,
    pub show_hidden: bool,
    pub font: FontChoice,
}

impl Default for Saved {
    fn default() -> Self {
        Self { theme_dark: true, show_hidden: false, font: FontChoice::NotoSansJp }
    }
}

fn settings_path() -> Option<PathBuf> {
    let dir = dirs::config_dir()?.join("filox");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("settings.txt"))
}

pub fn load() -> Saved {
    let Some(path) = settings_path() else { return Saved::default() };
    let Ok(text) = std::fs::read_to_string(path) else { return Saved::default() };
    let mut s = Saved::default();
    for line in text.lines() {
        let mut it = line.splitn(2, '=');
        match (it.next().map(str::trim), it.next().map(str::trim)) {
            (Some("theme"),       Some(v)) => s.theme_dark  = v == "dark",
            (Some("show_hidden"), Some(v)) => s.show_hidden  = v == "true",
            (Some("font"),        Some(v)) => s.font         = FontChoice::from_str(v),
            _ => {}
        }
    }
    s
}

pub fn save(theme_dark: bool, show_hidden: bool, font: &FontChoice) {
    let Some(path) = settings_path() else { return };
    let _ = std::fs::write(path, format!(
        "theme={}\nshow_hidden={}\nfont={}\n",
        if theme_dark { "dark" } else { "light" },
        show_hidden,
        font.as_str(),
    ));
}
