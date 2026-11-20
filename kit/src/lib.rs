//! Presets de fábrica y códec `.khodespreset`.
//!
//! Este crate define los datos durables de Khodes y valida el formato nativo.
//! No conoce el host, la UI ni el hilo de audio.
//!
//! Ver `docs/STATE_AND_PRESETS.md` y `docs/adr/0004-versioned-preset-codec.md`.

#![forbid(unsafe_code)]

/// Versión del layout binario de `.khodespreset`.
///
/// Independiente de la versión del producto y de `DSP_CONTRACT_VERSION`.
pub const PRESET_VERSION: u32 = 1;

/// Número de parámetros publicados, contrato estable `1..19`.
pub const PARAM_COUNT: usize = 19;

/// Longitud máxima del nombre, en bytes UTF-8.
pub const MAX_NAME_BYTES: usize = 63;

/// Tamaño máximo de un archivo `.khodespreset`.
pub const MAX_PRESET_BYTES: usize = 4096;

/// Magic del formato: ASCII `KHODESP` seguido de `0x00`.
pub const MAGIC: [u8; 8] = *b"KHODESP\0";

/// Tamaño mínimo: magic + versión + conteo + longitud + 19 × (id + valor) + CRC.
pub const MIN_PRESET_BYTES: usize = 8 + 4 + 4 + 4 + PARAM_COUNT * (4 + 8) + 4;

/// Un preset nativo: nombre UTF-8 y los 19 valores normalizados `0..1`.
#[derive(Clone, Debug, PartialEq)]
pub struct Preset {
    /// Nombre mostrado por el host. Se trunca a 63 bytes al codificar.
    pub name: String,
    /// Valores por ID: `values[id - 1]`.
    pub values: [f64; PARAM_COUNT],
}

/// Preset inmutable compilado en el producto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FactoryPreset {
    /// Nombre oficial del preset (`docs/STATE_AND_PRESETS.md` §3).
    pub name: &'static str,
    /// Valores normalizados por ID: `values[id - 1]`.
    pub values: [f64; PARAM_COUNT],
}

/// Los seis presets de fábrica, en el orden del combo del editor.
///
/// Los valores son la traducción exacta de los ajustes documentados a las
/// curvas de `src/params.rs`, que es el único punto autoritativo para el host.
/// Se fijan aquí como constantes porque `kit/` no puede depender del crate
/// raíz (dependencia circular).
pub const FACTORY_PRESETS: &[FactoryPreset] = &[
    // Tine Classic: todos los defaults de docs/PARAMETERS.md.
    FactoryPreset {
        name: "Tine Classic",
        values: [
            1.0,
            0.0,
            0.020100502512562814,
            0.07035175879396985,
            0.6,
            0.09547738693467336,
            0.3,
            0.0,
            0.5,
            0.5,
            0.4,
            0.0,
            0.16,
            0.0,
            0.0,
            0.4838709677419355,
            0.08333333333333333,
            0.5,
            0.5,
        ],
    },
    // Suitcase Warm: 8 kHz, Bass +3 dB, Treble −2 dB, chorus 0,3 Hz / 20 %,
    // reverb 15 %.
    FactoryPreset {
        name: "Suitcase Warm",
        values: [
            0.7516072988364303,
            0.0,
            0.020100502512562814,
            0.07035175879396985,
            0.6,
            0.09547738693467336,
            0.3,
            0.0,
            0.625,
            0.4166666666666667,
            0.4,
            0.0,
            0.06,
            0.2,
            0.15,
            0.4838709677419355,
            0.08333333333333333,
            0.5,
            0.5,
        ],
    },
    // Bell Air: 16 kHz, click 10 %, attack 2 ms, decay 0,4 s, sustain 30 %,
    // release 0,6 s, chorus 0,4 Hz / 25 %, reverb 35 %.
    FactoryPreset {
        name: "Bell Air",
        values: [
            0.939509123545538,
            0.1,
            0.005025125628140704,
            0.19597989949748745,
            0.3,
            0.2964824120603015,
            0.3,
            0.0,
            0.5,
            0.5,
            0.4,
            0.0,
            0.08,
            0.25,
            0.35,
            0.4838709677419355,
            0.08333333333333333,
            0.5,
            0.5,
        ],
    },
    // Dirty Drive: drive 60 %, 6 kHz, Bass +4 dB, Treble +2 dB.
    FactoryPreset {
        name: "Dirty Drive",
        values: [
            0.6736209953992307,
            0.0,
            0.020100502512562814,
            0.07035175879396985,
            0.6,
            0.09547738693467336,
            0.3,
            0.6,
            0.6666666666666666,
            0.5833333333333334,
            0.4,
            0.0,
            0.16,
            0.0,
            0.0,
            0.4838709677419355,
            0.08333333333333333,
            0.5,
            0.5,
        ],
    },
    // Tremolo Lounge: tremolo 3,5 Hz / 60 %, chorus 0,2 Hz / 15 %, reverb 20 %.
    FactoryPreset {
        name: "Tremolo Lounge",
        values: [
            1.0,
            0.0,
            0.020100502512562814,
            0.07035175879396985,
            0.6,
            0.09547738693467336,
            0.3,
            0.0,
            0.5,
            0.5,
            0.35,
            0.6,
            0.04,
            0.15,
            0.2,
            0.4838709677419355,
            0.08333333333333333,
            0.5,
            0.5,
        ],
    },
    // Silent Room: volume 0,25, 4 kHz, reverb 50 %, chorus 0,1 Hz / 10 %.
    FactoryPreset {
        name: "Silent Room",
        values: [
            0.5637054741273227,
            0.0,
            0.020100502512562814,
            0.07035175879396985,
            0.6,
            0.09547738693467336,
            0.25,
            0.0,
            0.5,
            0.5,
            0.4,
            0.0,
            0.02,
            0.1,
            0.5,
            0.4838709677419355,
            0.08333333333333333,
            0.5,
            0.5,
        ],
    },
];

/// Devuelve el preset de fábrica en `index`, o `None` si está fuera de rango.
#[must_use]
pub fn factory(index: usize) -> Option<&'static FactoryPreset> {
    FACTORY_PRESETS.get(index)
}

/// Busca un preset de fábrica por nombre exacto.
#[must_use]
pub fn factory_by_name(name: &str) -> Option<&'static FactoryPreset> {
    FACTORY_PRESETS.iter().find(|preset| preset.name == name)
}

/// Convierte un preset de fábrica en un preset nativo codificable.
#[must_use]
pub fn to_preset(source: &FactoryPreset) -> Preset {
    Preset {
        name: source.name.to_string(),
        values: source.values,
    }
}

/// Error de validación de un `.khodespreset`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// El archivo supera `MAX_PRESET_BYTES`.
    TooLarge,
    /// Faltan bytes para completar un campo.
    Truncated,
    /// Los primeros 8 bytes no son `MAGIC`.
    BadMagic,
    /// La versión no es `PRESET_VERSION`.
    UnsupportedVersion(u32),
    /// `param_count` no es 19.
    BadParamCount(u32),
    /// `name_len` es mayor que 63.
    BadNameLen(u32),
    /// El nombre no es UTF-8 válido.
    InvalidUtf8,
    /// ID desconocido o fuera de `1..19`.
    BadId(u32),
    /// Un ID aparece más de una vez.
    DuplicateId(u32),
    /// Valor `NaN`, infinito o fuera de `0..1` para ese ID.
    InvalidValue(u32),
    /// CRC-32 incorrecto.
    BadCrc,
    /// Hay bytes después del CRC.
    TrailingBytes,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge => write!(f, "preset mayor a {MAX_PRESET_BYTES} bytes"),
            Self::Truncated => write!(f, "preset truncado"),
            Self::BadMagic => write!(f, "magic inválido"),
            Self::UnsupportedVersion(v) => write!(f, "versión de preset no soportada: {v}"),
            Self::BadParamCount(n) => write!(f, "param_count inválido: {n}"),
            Self::BadNameLen(n) => write!(f, "name_len inválido: {n}"),
            Self::InvalidUtf8 => write!(f, "nombre no es UTF-8 válido"),
            Self::BadId(id) => write!(f, "ID de parámetro inválido: {id}"),
            Self::DuplicateId(id) => write!(f, "ID de parámetro duplicado: {id}"),
            Self::InvalidValue(id) => write!(f, "valor inválido para el parámetro {id}"),
            Self::BadCrc => write!(f, "CRC-32 incorrecto"),
            Self::TrailingBytes => write!(f, "bytes sobrantes después del CRC"),
        }
    }
}

impl std::error::Error for Error {}

/// CRC-32/ISO-HDLC (polinomio reflejado `0xEDB88320`, init/final XOR `0xFFFFFFFF`).
///
/// Detecta corrupción accidental; no es una firma criptográfica.
#[must_use]
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Trunca un nombre a `MAX_NAME_BYTES` sin cortar un carácter UTF-8.
#[must_use]
fn truncate_name(name: &str) -> &str {
    if name.len() <= MAX_NAME_BYTES {
        return name;
    }
    let mut end = MAX_NAME_BYTES;
    while end > 0 && !name.is_char_boundary(end) {
        end -= 1;
    }
    &name[..end]
}

/// Codifica un preset v1 little-endian, sin padding ni bytes extra.
///
/// # Errors
///
/// Devuelve [`Error::InvalidValue`] si algún valor no es finito o está fuera
/// de `0..1`. El nombre se trunca a 63 bytes sin partir un carácter UTF-8.
pub fn encode(preset: &Preset) -> Result<Vec<u8>, Error> {
    for (index, value) in preset.values.iter().enumerate() {
        if !value.is_finite() || !(0.0..=1.0).contains(value) {
            return Err(Error::InvalidValue(index as u32 + 1));
        }
    }
    let name = truncate_name(&preset.name);
    let mut buf = Vec::with_capacity(20 + MAX_NAME_BYTES + PARAM_COUNT * 12 + 4);
    buf.extend_from_slice(&MAGIC);
    buf.extend_from_slice(&PRESET_VERSION.to_le_bytes());
    buf.extend_from_slice(&(PARAM_COUNT as u32).to_le_bytes());
    buf.extend_from_slice(&(name.len() as u32).to_le_bytes());
    buf.extend_from_slice(name.as_bytes());
    for (id, value) in preset.values.iter().enumerate() {
        buf.extend_from_slice(&((id + 1) as u32).to_le_bytes());
        buf.extend_from_slice(&value.to_le_bytes());
    }
    let checksum = crc32(&buf);
    buf.extend_from_slice(&checksum.to_le_bytes());
    debug_assert!(buf.len() <= MAX_PRESET_BYTES);
    Ok(buf)
}

/// Lector acotado que falla sin `panic!` ante cualquier truncamiento.
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], Error> {
        let end = self.pos.checked_add(count).ok_or(Error::Truncated)?;
        let slice = self.bytes.get(self.pos..end).ok_or(Error::Truncated)?;
        self.pos = end;
        Ok(slice)
    }

    fn u32(&mut self) -> Result<u32, Error> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn f64(&mut self) -> Result<f64, Error> {
        let bytes = self.take(8)?;
        let mut array = [0_u8; 8];
        array.copy_from_slice(bytes);
        Ok(f64::from_le_bytes(array))
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }
}

/// Decodifica un preset v1 con validación estricta y carga atómica.
///
/// O se valida todo y se devuelve un [`Preset`], o se devuelve un [`Error`] sin
/// aplicar cambio alguno. Se rechazan magic, versión, conteo, longitud y UTF-8
/// del nombre, IDs duplicados/desconocidos/fuera de rango, valores no finitos o
/// fuera de `0..1`, truncamiento, bytes sobrantes, CRC y tamaño excesivo.
pub fn decode(bytes: &[u8]) -> Result<Preset, Error> {
    if bytes.len() > MAX_PRESET_BYTES {
        return Err(Error::TooLarge);
    }
    let mut reader = Reader::new(bytes);
    if reader.take(MAGIC.len())? != MAGIC {
        return Err(Error::BadMagic);
    }
    let version = reader.u32()?;
    if version != PRESET_VERSION {
        return Err(Error::UnsupportedVersion(version));
    }
    let count = reader.u32()?;
    if count as usize != PARAM_COUNT {
        return Err(Error::BadParamCount(count));
    }
    let name_len = reader.u32()? as usize;
    if name_len > MAX_NAME_BYTES {
        return Err(Error::BadNameLen(name_len as u32));
    }
    let name_bytes = reader.take(name_len)?;
    let name = std::str::from_utf8(name_bytes).map_err(|_| Error::InvalidUtf8)?;

    let mut values = [0.0_f64; PARAM_COUNT];
    let mut seen = [false; PARAM_COUNT];
    for _ in 0..PARAM_COUNT {
        let id = reader.u32()?;
        let index = usize::try_from(id)
            .ok()
            .and_then(|id| id.checked_sub(1))
            .filter(|index| *index < PARAM_COUNT)
            .ok_or(Error::BadId(id))?;
        if seen[index] {
            return Err(Error::DuplicateId(id));
        }
        let value = reader.f64()?;
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(Error::InvalidValue(id));
        }
        seen[index] = true;
        values[index] = value;
    }

    // CRC sobre todo lo anterior, sin incluir el propio CRC ni bytes extra.
    let payload_end = reader.pos;
    let stored_crc = reader.u32()?;
    if reader.remaining() != 0 {
        return Err(Error::TrailingBytes);
    }
    if crc32(&bytes[..payload_end]) != stored_crc {
        return Err(Error::BadCrc);
    }

    Ok(Preset {
        name: name.to_string(),
        values,
    })
}
