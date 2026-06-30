use egui::{FontData, FontDefinitions, FontFamily};

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

pub fn setup(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // Meiryo — covers both Latin and Japanese, loaded from Windows system fonts.
    // Inserted at position 0 so it takes priority over all bundled fonts.
    let meiryo_paths: &[(&str, u32)] = &[
        ("C:\\Windows\\Fonts\\meiryo.ttc", 0),
        ("C:\\Windows\\Fonts\\MEIRYO.TTC", 0),
    ];
    let mut meiryo_loaded = false;
    for (path, index) in meiryo_paths {
        match std::fs::read(path) {
            Ok(data) => {
                crate::log_info!("Meiryo loaded: {}", path);
                let mut fd = FontData::from_owned(data);
                fd.index = *index;
                fonts.font_data.insert("Meiryo".to_owned(), fd);
                fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "Meiryo".to_owned());
                fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "Meiryo".to_owned());
                meiryo_loaded = true;
                break;
            }
            Err(e) => { crate::log_info!("Meiryo skip: {} — {}", path, e); }
        }
    }
    if !meiryo_loaded {
        crate::log_info!("WARNING: Meiryo not found; falling back to bundled fonts");
    }

    // IBM Plex Sans — fallback for Latin glyphs Meiryo may not cover
    if !IBMPLEX_BYTES.is_empty() {
        fonts.font_data.insert("IBMPlexSans".to_owned(), FontData::from_static(IBMPLEX_BYTES));
        fonts.families.entry(FontFamily::Proportional).or_default().push("IBMPlexSans".to_owned());
    }

    // JetBrains Mono — fallback for monospace
    if !JETBRAINS_BYTES.is_empty() {
        fonts.font_data.insert("JetBrainsMono".to_owned(), FontData::from_static(JETBRAINS_BYTES));
        fonts.families.entry(FontFamily::Monospace).or_default().push("JetBrainsMono".to_owned());
    }

    // Noto Sans JP — fallback for CJK glyphs not in Meiryo
    if !NOTOSANSJP_BYTES.is_empty() {
        fonts.font_data.insert("NotoSansJP".to_owned(), FontData::from_static(NOTOSANSJP_BYTES));
        fonts.families.entry(FontFamily::Proportional).or_default().push("NotoSansJP".to_owned());
        fonts.families.entry(FontFamily::Monospace).or_default().push("NotoSansJP".to_owned());
    }

    // Material Symbols — always last so icon PUA glyphs don't shadow text
    if !MATERIAL_BYTES.is_empty() {
        fonts.font_data.insert("MaterialSymbols".to_owned(), FontData::from_static(MATERIAL_BYTES));
        fonts.families.entry(FontFamily::Proportional).or_default().push("MaterialSymbols".to_owned());
        fonts.families.entry(FontFamily::Monospace).or_default().push("MaterialSymbols".to_owned());
    }

    ctx.set_fonts(fonts);
}
