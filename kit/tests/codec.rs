//! Pruebas del códec `.khodespreset` v1 (`docs/STATE_AND_PRESETS.md` §4 y §9).

use khodes_kit::{
    Error, FACTORY_PRESETS, MAGIC, MAX_NAME_BYTES, MAX_PRESET_BYTES, PARAM_COUNT, PRESET_VERSION,
    crc32, decode, encode, factory, factory_by_name, to_preset,
};

/// Recalcula el CRC de un blob modificado para que los tests de validación
/// estructural no se confundan con un CRC inválido.
fn with_valid_crc(mut bytes: Vec<u8>) -> Vec<u8> {
    let len = bytes.len();
    let checksum = crc32(&bytes[..len - 4]);
    bytes[len - 4..].copy_from_slice(&checksum.to_le_bytes());
    bytes
}

fn patch_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn patch_f64(bytes: &mut [u8], offset: usize, value: f64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn baseline() -> Vec<u8> {
    encode(&to_preset(factory(0).expect("Tine Classic"))).expect("encode válido")
}

#[test]
fn crc32_matches_the_standard_vector() {
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(crc32(b""), 0);
}

#[test]
fn round_trip_all_19_parameters() {
    let values = std::array::from_fn(|i| (i + 1) as f64 / 20.0);
    let preset = khodes_kit::Preset {
        name: "RoundTrip".to_string(),
        values,
    };
    let bytes = encode(&preset).expect("encode");
    assert_eq!(decode(&bytes).expect("decode"), preset);

    let bytes = decode(&bytes).expect("decode dos veces");
    assert_eq!(bytes.values.len(), PARAM_COUNT);
    for (id, value) in bytes.values.iter().enumerate() {
        assert_eq!(*value, (id + 1) as f64 / 20.0, "ID {}", id + 1);
    }
}

#[test]
fn round_trip_all_six_factory_presets() {
    assert_eq!(FACTORY_PRESETS.len(), 6);
    for source in FACTORY_PRESETS {
        let bytes = encode(&to_preset(source)).expect("encode de preset de fábrica");
        let decoded = decode(&bytes).expect("decode de preset de fábrica");
        assert_eq!(decoded.name, source.name);
        for (id, (got, expected)) in decoded.values.iter().zip(source.values.iter()).enumerate() {
            assert_eq!(
                got.to_bits(),
                expected.to_bits(),
                "{} / ID {}",
                source.name,
                id + 1
            );
        }
    }
}

#[test]
fn round_trip_extreme_values() {
    for value in [0.0_f64, 1.0] {
        let preset = khodes_kit::Preset {
            name: "Extremos".to_string(),
            values: [value; PARAM_COUNT],
        };
        let decoded = decode(&encode(&preset).expect("encode")).expect("decode");
        assert_eq!(decoded, preset);
    }
}

#[test]
fn round_trip_multibyte_and_maximum_length_names() {
    for name in ["Tine ♥ テスト 🎹", &"a".repeat(MAX_NAME_BYTES), "€", ""] {
        let preset = khodes_kit::Preset {
            name: name.to_string(),
            values: [0.5; PARAM_COUNT],
        };
        let decoded = decode(&encode(&preset).expect("encode")).expect("decode");
        assert_eq!(decoded.name, name);
    }
}

#[test]
fn encoding_truncates_long_names_without_splitting_a_character() {
    // 62 bytes ASCII + heart (3 bytes): el corazón no cabe en el byte 63.
    let mut name = "a".repeat(62);
    name.push('♥');
    name.push_str("cola");
    let preset = khodes_kit::Preset {
        name,
        values: [0.5; PARAM_COUNT],
    };
    let bytes = encode(&preset).expect("encode");
    let decoded = decode(&bytes).expect("decode");
    assert_eq!(decoded.name.len(), 62);
    assert_eq!(decoded.name, "a".repeat(62));

    let long_ascii = "x".repeat(100);
    let decoded = decode(
        &encode(&khodes_kit::Preset {
            name: long_ascii,
            values: [0.5; PARAM_COUNT],
        })
        .expect("encode"),
    )
    .expect("decode");
    assert_eq!(decoded.name.len(), MAX_NAME_BYTES);
}

#[test]
fn rejects_bad_magic_version_count_and_name_len() {
    let valid = baseline();

    let mut bad_magic = valid.clone();
    bad_magic[0] = b'X';
    assert_eq!(decode(&bad_magic), Err(Error::BadMagic));

    let mut bad_version = valid.clone();
    patch_u32(&mut bad_version, 8, PRESET_VERSION + 1);
    assert_eq!(
        decode(&with_valid_crc(bad_version)),
        Err(Error::UnsupportedVersion(2))
    );

    let mut bad_count = valid.clone();
    patch_u32(&mut bad_count, 12, 18);
    assert_eq!(
        decode(&with_valid_crc(bad_count)),
        Err(Error::BadParamCount(18))
    );

    let mut bad_len = valid.clone();
    patch_u32(&mut bad_len, 16, 64);
    assert_eq!(decode(&with_valid_crc(bad_len)), Err(Error::BadNameLen(64)));
}

#[test]
fn rejects_invalid_utf8_name() {
    let mut bytes = baseline();
    // "Tine Classic" empieza en el offset 20.
    bytes[20] = 0xFF;
    assert_eq!(decode(&with_valid_crc(bytes)), Err(Error::InvalidUtf8));
}

#[test]
fn rejects_bad_duplicate_and_out_of_range_ids() {
    let valid = baseline();
    let ids_at = 20 + "Tine Classic".len();

    let mut bad_id = valid.clone();
    patch_u32(&mut bad_id, ids_at, 20);
    assert_eq!(decode(&with_valid_crc(bad_id)), Err(Error::BadId(20)));

    let mut zero_id = valid.clone();
    patch_u32(&mut zero_id, ids_at, 0);
    assert_eq!(decode(&with_valid_crc(zero_id)), Err(Error::BadId(0)));

    let mut duplicate = valid.clone();
    patch_u32(&mut duplicate, ids_at + 12, 1);
    assert_eq!(
        decode(&with_valid_crc(duplicate)),
        Err(Error::DuplicateId(1))
    );
}

#[test]
fn rejects_nan_infinite_and_out_of_range_values() {
    let valid = baseline();
    let ids_at = 20 + "Tine Classic".len();
    for (value, label) in [
        (f64::NAN, "NaN"),
        (f64::INFINITY, "inf"),
        (f64::NEG_INFINITY, "-inf"),
        (-0.001, "menor que 0"),
        (1.001, "mayor que 1"),
    ] {
        let mut bytes = valid.clone();
        patch_f64(&mut bytes, ids_at + 4, value);
        assert_eq!(
            decode(&with_valid_crc(bytes)),
            Err(Error::InvalidValue(1)),
            "{label}"
        );
    }

    let preset = khodes_kit::Preset {
        name: "malo".to_string(),
        values: [f64::NAN; PARAM_COUNT],
    };
    assert_eq!(encode(&preset), Err(Error::InvalidValue(1)));
}

#[test]
fn rejects_every_truncation() {
    let valid = baseline();
    for len in 0..valid.len() {
        assert!(
            decode(&valid[..len]).is_err(),
            "una longitud de {len} bytes debe rechazarse"
        );
    }
    assert!(decode(&valid).is_ok());
}

#[test]
fn rejects_trailing_bytes() {
    let mut bytes = baseline();
    bytes.push(0);
    assert_eq!(decode(&bytes), Err(Error::TrailingBytes));
}

#[test]
fn rejects_bad_crc() {
    let mut bytes = baseline();
    let last = bytes.len() - 1;
    bytes[last] ^= 0x01;
    assert_eq!(decode(&bytes), Err(Error::BadCrc));
}

#[test]
fn rejects_oversized_file() {
    assert_eq!(
        decode(&vec![0_u8; MAX_PRESET_BYTES + 1]),
        Err(Error::TooLarge)
    );
}

#[test]
fn encoding_is_byte_stable() {
    let preset = to_preset(factory(2).expect("Bell Air"));
    assert_eq!(
        encode(&preset).expect("encode"),
        encode(&preset).expect("encode")
    );
}

#[test]
fn factory_lookup_is_stable() {
    assert_eq!(factory(0).map(|p| p.name), Some("Tine Classic"));
    assert_eq!(factory(5).map(|p| p.name), Some("Silent Room"));
    assert!(factory(6).is_none());
    assert_eq!(
        factory_by_name("Dirty Drive").map(|p| p.name),
        Some("Dirty Drive")
    );
    assert!(factory_by_name("no existe").is_none());
    assert_eq!(MAGIC, *b"KHODESP\0");
}
