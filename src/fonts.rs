use egui::{FontData, FontDefinitions, FontFamily};
use crate::settings::FontChoice;

#[cfg(have_material_font)]
const MATERIAL_BYTES: &[u8] = include_bytes!("../assets/fonts/MaterialSymbolsRounded.ttf");
#[cfg(not(have_material_font))]
const MATERIAL_BYTES: &[u8] = &[];

#[cfg(have_jetbrains_font)]
const JETBRAINS_BYTES: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
#[cfg(not(have_jetbrains_font))]
const JETBRAINS_BYTES: &[u8] = &[];

#[cfg(have_ibmplex_font)]
const IBMPLEX_BYTES: &[u8] = include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf");
#[cfg(not(have_ibmplex_font))]
const IBMPLEX_BYTES: &[u8] = &[];

#[cfg(have_notosansjp_font)]
const NOTOSANSJP_BYTES: &[u8] = include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf");
#[cfg(not(have_notosansjp_font))]
const NOTOSANSJP_BYTES: &[u8] = &[];

pub fn setup(ctx: &egui::Context, choice: &FontChoice) {
    let mut fonts = FontDefinitions::default();

    // Primary font — chosen by user, inserted at position 0
    match choice {
        FontChoice::Meiryo => {
            load_system(&mut fonts, &[
                ("C:\\Windows\\Fonts\\meiryo.ttc",  0),
                ("C:\\Windows\\Fonts\\MEIRYO.TTC",  0),
            ]);
        }
        FontChoice::YuGothic => {
            load_system(&mut fonts, &[
                ("C:\\Windows\\Fonts\\YuGothR.ttc", 0),
                ("C:\\Windows\\Fonts\\YuGothM.ttc", 0),
            ]);
        }
        FontChoice::BizUdGothic => {
            load_system(&mut fonts, &[
                ("C:\\Windows\\Fonts\\BIZ-UDGothicR.ttc", 0),
            ]);
        }
        FontChoice::MsGothic => {
            load_system(&mut fonts, &[
                ("C:\\Windows\\Fonts\\msgothic.ttc", 0),
            ]);
        }
        FontChoice::NotoSansJp => {
            if !NOTOSANSJP_BYTES.is_empty() {
                fonts.font_data.insert("PrimaryFont".to_owned(), FontData::from_static(NOTOSANSJP_BYTES));
                fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "PrimaryFont".to_owned());
                fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "PrimaryFont".to_owned());
                crate::log_info!("Primary font: bundled NotoSansJP");
            }
        }
        FontChoice::IbmPlexSans => {
            if !IBMPLEX_BYTES.is_empty() {
                fonts.font_data.insert("IBMPlexSans".to_owned(), FontData::from_static(IBMPLEX_BYTES));
                fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "IBMPlexSans".to_owned());
                fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "IBMPlexSans".to_owned());
                crate::log_info!("Primary font: bundled IBMPlexSans");
            }
        }
    }

    // JetBrains Mono — monospace fallback
    if !JETBRAINS_BYTES.is_empty() {
        fonts.font_data.insert("JetBrainsMono".to_owned(), FontData::from_static(JETBRAINS_BYTES));
        fonts.families.entry(FontFamily::Monospace).or_default().push("JetBrainsMono".to_owned());
    }

    // IBM Plex Sans — Latin fallback (skip if already primary)
    if !matches!(choice, FontChoice::IbmPlexSans) && !IBMPLEX_BYTES.is_empty() {
        fonts.font_data.insert("IBMPlexSans".to_owned(), FontData::from_static(IBMPLEX_BYTES));
        fonts.families.entry(FontFamily::Proportional).or_default().push("IBMPlexSans".to_owned());
    }

    // Noto Sans JP — CJK fallback (skip if already primary)
    if !matches!(choice, FontChoice::NotoSansJp) && !NOTOSANSJP_BYTES.is_empty() {
        fonts.font_data.insert("NotoSansJP".to_owned(), FontData::from_static(NOTOSANSJP_BYTES));
        fonts.families.entry(FontFamily::Proportional).or_default().push("NotoSansJP".to_owned());
        fonts.families.entry(FontFamily::Monospace).or_default().push("NotoSansJP".to_owned());
    }

    // Material Symbols — always last so PUA glyphs don't shadow text
    if !MATERIAL_BYTES.is_empty() {
        fonts.font_data.insert("MaterialSymbols".to_owned(), FontData::from_static(MATERIAL_BYTES));
        fonts.families.entry(FontFamily::Proportional).or_default().push("MaterialSymbols".to_owned());
        fonts.families.entry(FontFamily::Monospace).or_default().push("MaterialSymbols".to_owned());
    }

    ctx.set_fonts(fonts);
}

fn load_system(fonts: &mut FontDefinitions, candidates: &[(&str, u32)]) {
    for (path, index) in candidates {
        match std::fs::read(path) {
            Ok(data) => {
                crate::log_info!("Primary font loaded: {}", path);
                let mut fd = FontData::from_owned(data);
                fd.index = *index;
                fonts.font_data.insert("PrimaryFont".to_owned(), fd);
                fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "PrimaryFont".to_owned());
                fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "PrimaryFont".to_owned());
                return;
            }
            Err(e) => { crate::log_info!("Font skip: {} — {}", path, e); }
        }
    }
    crate::log_info!("WARNING: no system font found for this choice");
}
