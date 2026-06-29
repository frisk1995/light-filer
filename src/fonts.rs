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

// Windows system fonts with Japanese glyph coverage (tried in order)
const JP_FONT_CANDIDATES: &[(&str, u32)] = &[
    ("C:\\Windows\\Fonts\\meiryo.ttc", 0),
    ("C:\\Windows\\Fonts\\YuGothR.ttc", 0),
    ("C:\\Windows\\Fonts\\YuGothM.ttc", 0),
    ("C:\\Windows\\Fonts\\msgothic.ttc", 0),
    ("C:\\Windows\\Fonts\\msmincho.ttc", 0),
];

pub fn setup(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // IBM Plex Sans (UI proportional)
    if !IBMPLEX_BYTES.is_empty() {
        fonts.font_data.insert(
            "IBMPlexSans".to_owned(),
            FontData::from_static(IBMPLEX_BYTES),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "IBMPlexSans".to_owned());
    }

    // JetBrains Mono (monospace)
    if !JETBRAINS_BYTES.is_empty() {
        fonts.font_data.insert(
            "JetBrainsMono".to_owned(),
            FontData::from_static(JETBRAINS_BYTES),
        );
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "JetBrainsMono".to_owned());
    }

    // Japanese system font — load at runtime, append as fallback
    for (path, index) in JP_FONT_CANDIDATES {
        if let Ok(data) = std::fs::read(path) {
            let mut fd = FontData::from_owned(data);
            fd.index = *index;
            fonts.font_data.insert("JapaneseFont".to_owned(), fd);
            // Append after Latin fonts so CJK glyphs are resolved as fallback
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push("JapaneseFont".to_owned());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push("JapaneseFont".to_owned());
            break;
        }
    }

    // Material Symbols (icon PUA glyphs) — must come after JP font
    if !MATERIAL_BYTES.is_empty() {
        fonts.font_data.insert(
            "MaterialSymbols".to_owned(),
            FontData::from_static(MATERIAL_BYTES),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("MaterialSymbols".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("MaterialSymbols".to_owned());
    }

    ctx.set_fonts(fonts);
}
