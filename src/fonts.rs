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

// Fallback: Windows system fonts tried only when the bundled font is absent.
#[cfg(not(have_notosansjp_font))]
const JP_FONT_CANDIDATES: &[(&str, u32)] = &[
    ("C:\\Windows\\Fonts\\NotoSansJP-VF.ttf", 0),
    ("C:\\Windows\\Fonts\\BIZ-UDGothicR.ttc", 0),
    ("C:\\Windows\\Fonts\\meiryo.ttc", 0),
    ("C:\\Windows\\Fonts\\YuGothR.ttc", 0),
    ("C:\\Windows\\Fonts\\YuGothM.ttc", 0),
    ("C:\\Windows\\Fonts\\msgothic.ttc", 0),
    ("C:\\Windows\\Fonts\\msmincho.ttc", 0),
];

pub fn setup(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // IBM Plex Sans (UI proportional) — inserted at position 0
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

    // JetBrains Mono (monospace) — inserted at position 0
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

    let jp_insert_pos: usize = if !IBMPLEX_BYTES.is_empty() { 1 } else { 0 };
    let jp_mono_pos: usize = if !JETBRAINS_BYTES.is_empty() { 1 } else { 0 };

    // Bundled Noto Sans JP — works on any OS
    if !NOTOSANSJP_BYTES.is_empty() {
        fonts.font_data.insert(
            "NotoSansJP".to_owned(),
            FontData::from_static(NOTOSANSJP_BYTES),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(jp_insert_pos, "NotoSansJP".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(jp_mono_pos, "NotoSansJP".to_owned());
        crate::log_info!("Japanese font: using bundled NotoSansJP-Regular");
    } else {
        // Fallback to Windows system fonts when the bundled font is not present
        #[cfg(not(have_notosansjp_font))]
        {
            let mut jp_loaded = false;
            for (path, index) in JP_FONT_CANDIDATES {
                match std::fs::read(path) {
                    Ok(data) => {
                        crate::log_info!("Japanese font loaded: {} (index {})", path, index);
                        let mut fd = FontData::from_owned(data);
                        fd.index = *index;
                        fonts.font_data.insert("JapaneseFont".to_owned(), fd);
                        fonts
                            .families
                            .entry(FontFamily::Proportional)
                            .or_default()
                            .insert(jp_insert_pos, "JapaneseFont".to_owned());
                        fonts
                            .families
                            .entry(FontFamily::Monospace)
                            .or_default()
                            .insert(jp_mono_pos, "JapaneseFont".to_owned());
                        jp_loaded = true;
                        break;
                    }
                    Err(e) => {
                        crate::log_info!("Japanese font skip: {} — {}", path, e);
                    }
                }
            }
            if !jp_loaded {
                crate::log_info!("WARNING: no Japanese font loaded; CJK glyphs will be missing");
            }
        }
    }

    // Material Symbols (icon PUA glyphs) — always last so PUA does not shadow CJK
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
