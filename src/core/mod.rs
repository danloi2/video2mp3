pub mod types;
pub mod probe;
pub mod convert;

pub use types::{PistaAudio, TipoConversion, OpcionesVideo, AceleracionHW, ProgressUpdate, InfoMedia};
pub use probe::{verificar_ffmpeg, verificar_ytdlp, elegir_pista_defecto, obtener_pistas, obtener_version_ffmpeg, CapacidadesHardware, detectar_capacidades_hardware, obtener_info_media};
pub use convert::{convertir_archivo, descargar_youtube};
