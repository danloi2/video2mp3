use crate::core::PistaAudio;

pub fn cr(v: u8) -> eframe::egui::CornerRadius {
    eframe::egui::CornerRadius { nw: v, ne: v, sw: v, se: v }
}

pub fn nombre_idioma(codigo: &str) -> String {
    match codigo.to_lowercase().as_str() {
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
        "desconocido"        => "Desconocido",
        other                => other,
    }.to_string()
}

pub fn etiqueta_pista(pista: &PistaAudio, num: usize) -> String {
    format!(
        "Pista {} — {} ({})",
        num + 1,
        nombre_idioma(&pista.idioma),
        pista.codec.to_uppercase()
    )
}
