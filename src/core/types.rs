#[derive(Clone, Debug, Copy, PartialEq)]
pub enum TipoConversion {
    AudioMP3,
    VideoH264,
    VideoH265,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum AceleracionHW {
    Ninguna,
    NVENC,
    QSV,
    AMF,
    VAAPI,
    VideoToolbox,
}

#[derive(Clone, Debug, Copy)]
pub struct OpcionesVideo {
    pub preservar_grano: bool,
    pub optimizar_color: bool,
    pub aceleracion:     AceleracionHW,
}

#[derive(Clone, Debug)]
pub struct PistaAudio {
    pub indice_stream: u64,
    pub codec: String,
    pub idioma: String,
}

#[derive(Clone, Debug, Copy)]
pub enum ProgressUpdate {
    Ratio(f32),
    Playlist(usize, usize), // (actual, total)
}

