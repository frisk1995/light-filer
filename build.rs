fn main() {
    println!("cargo:rerun-if-changed=assets/fonts/");
    println!("cargo::rustc-check-cfg=cfg(have_material_font)");
    println!("cargo::rustc-check-cfg=cfg(have_jetbrains_font)");
    println!("cargo::rustc-check-cfg=cfg(have_ibmplex_font)");
    println!("cargo::rustc-check-cfg=cfg(have_notosansjp_font)");
    let fonts_dir = std::path::Path::new("assets/fonts");
    if fonts_dir.join("MaterialSymbolsRounded.ttf").exists() {
        println!("cargo:rustc-cfg=have_material_font");
    }
    if fonts_dir.join("JetBrainsMono-Regular.ttf").exists() {
        println!("cargo:rustc-cfg=have_jetbrains_font");
    }
    if fonts_dir.join("IBMPlexSans-Regular.ttf").exists() {
        println!("cargo:rustc-cfg=have_ibmplex_font");
    }
    if fonts_dir.join("NotoSansJP-Regular.ttf").exists() {
        println!("cargo:rustc-cfg=have_notosansjp_font");
    }
}
