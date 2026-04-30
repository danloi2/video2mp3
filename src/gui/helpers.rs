use crate::core::AudioTrack;

/// Shorthand to create a uniform `CornerRadius` for all four corners.
pub fn cr(v: u8) -> eframe::egui::CornerRadius {
    eframe::egui::CornerRadius { nw: v, ne: v, sw: v, se: v }
}

/// Converts an ISO 639-1/2 language code into its full display name in Spanish.
pub fn get_language_name(code: &str) -> String {
    match code.to_lowercase().as_str() {
        "spa" | "es" | "esp" => "Español",
        "eng" | "en"         => "Inglés",
        "fre" | "fra" | "fr" => "Francés",
        "ger" | "deu" | "de" => "Alemán",
        "ita" | "it"         => "Italiano",
        "por" | "pt"         => "Portugués",
        "jpn" | "ja"         => "Japonés",
        "chi" | "zho" | "zh" => "Chino",
        "kor" | "ko"         => "Coreano",
        "ara" | "ar"         => "Árabe",
        "rus" | "ru"         => "Ruso",
        "cat" | "ca"         => "Catalán",
        "eus" | "eu"         => "Euskera",
        "glg" | "gl"         => "Gallego",
        "unknown" | "desconocido" => "Desconocido",
        other                => other,
    }.to_string()
}

/// Formats an audio track into a user-friendly label for selection menus.
pub fn get_track_label(track: &AudioTrack, num: usize) -> String {
    format!(
        "Track {} — {} ({})",
        num + 1,
        get_language_name(&track.language),
        track.codec.to_uppercase()
    )
}
