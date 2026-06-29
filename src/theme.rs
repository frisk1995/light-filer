use egui::Color32;

#[derive(Clone, Copy, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Accent {
    Rust,
    Aqua,
    Green,
    Purple,
}

impl Accent {
    pub fn color(self) -> Color32 {
        match self {
            Accent::Rust => Color32::from_rgb(0xe0, 0x82, 0x4a),
            Accent::Aqua => Color32::from_rgb(0x5f, 0x9e, 0x95),
            Accent::Green => Color32::from_rgb(0x8a, 0xa8, 0x61),
            Accent::Purple => Color32::from_rgb(0xc0, 0x7a, 0x92),
        }
    }
    pub fn soft(self) -> Color32 {
        let c = self.color();
        Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 44)
    }
}

#[derive(Clone)]
pub struct Tokens {
    pub bg: Color32,
    pub titlebar: Color32,
    pub list: Color32,
    pub border: Color32,
    pub hover: Color32,
    pub elev: Color32,
    pub text: Color32,
    pub dim: Color32,
    pub faint: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,

    // syntax highlight
    pub syn_keyword: Color32,
    pub syn_type: Color32,
    pub syn_func: Color32,
    pub syn_string: Color32,
}

impl Tokens {
    pub fn new(theme: Theme, accent: Accent) -> Self {
        let (bg, titlebar, list, border, hover, elev, text, dim, faint) = match theme {
            Theme::Dark => (
                Color32::from_rgb(0x16, 0x18, 0x1b),
                Color32::from_rgb(0x1b, 0x1e, 0x21),
                Color32::from_rgb(0x12, 0x14, 0x17),
                Color32::from_rgb(0x28, 0x2c, 0x31),
                Color32::from_rgb(0x1e, 0x22, 0x26),
                Color32::from_rgb(0x23, 0x27, 0x2c),
                Color32::from_rgb(0xdc, 0xdf, 0xe2),
                Color32::from_rgb(0x88, 0x8f, 0x95),
                Color32::from_rgb(0x56, 0x5d, 0x63),
            ),
            Theme::Light => (
                Color32::from_rgb(0xee, 0xf0, 0xf1),
                Color32::from_rgb(0xe4, 0xe7, 0xe9),
                Color32::from_rgb(0xfb, 0xfc, 0xfc),
                Color32::from_rgb(0xd8, 0xdc, 0xdf),
                Color32::from_rgb(0xe9, 0xed, 0xee),
                Color32::from_rgb(0xff, 0xff, 0xff),
                Color32::from_rgb(0x27, 0x2b, 0x2e),
                Color32::from_rgb(0x69, 0x70, 0x77),
                Color32::from_rgb(0x9a, 0xa1, 0xa7),
            ),
        };

        let a = accent.color();
        let syn_keyword = Color32::from_rgb(0xc0, 0x7a, 0x92);
        let syn_type = Color32::from_rgb(0xc9, 0x9a, 0x4e);
        let syn_func = Color32::from_rgb(0x5f, 0x9e, 0x95);
        let syn_string = Color32::from_rgb(0x8a, 0xa8, 0x61);

        Tokens {
            bg,
            titlebar,
            list,
            border,
            hover,
            elev,
            text,
            dim,
            faint,
            accent: a,
            accent_soft: accent.soft(),
            syn_keyword,
            syn_type,
            syn_func,
            syn_string,
        }
    }
}
