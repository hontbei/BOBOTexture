use xxhash_rust::xxh64::xxh64;

/// Public: 13 fixture sprite names, lex order, used by harness assertions and downstream T11.
pub const FIXTURE_SPRITE_NAMES: [&str; 13] = [
    "Launcher_AA_lv1",
    "Launcher_AA_lv2",
    "Launcher_AA_lv3",
    "Launcher_AA_lv4",
    "Launcher_AA_lv5",
    "Launcher_AA_lv6",
    "Launcher_AA_lv7",
    "Launcher_AA_lv8",
    "Launcher_AB_lv1",
    "Launcher_AB_lv2",
    "Launcher_AB_lv3",
    "Launcher_AB_lv4",
    "Launcher_AB_lv5",
];

/// Reproduces Unity 2022.3 AssetImporter.MakeLocalFileIDWithHash(213, name, 0).
/// Algorithm (from Oracle consultation, validated against fixture):
///   input  = format!("Type:Sprite->{}{}", name, 0)   // offset literal 0
///   bytes  = input.as_bytes()                         // UTF-8
///   h      = xxhash_rust::xxh64::xxh64(bytes, 0)      // canonical xxHash64, seed 0
///   id     = h as i64                                 // reinterpret as signed two's complement
pub fn unity_sprite_internal_id(sprite_name: &str) -> i64 {
    let input = format!("Type:Sprite->{}{}", sprite_name, 0);
    xxh64(input.as_bytes(), 0) as i64
}

/// Reproduces Unity GUID.CreateGUIDFromSInt64(internal_id).
/// Format observed in fixture: 32 lowercase hex chars.
///   bytes = (internal_id as u64).to_le_bytes()       // 8 bytes, little-endian
///   for each byte b: emit hex(b & 0x0f) then hex((b >> 4) & 0x0f)   // low-nibble first
///   then append the literal suffix "0800000000000000"
pub fn unity_guid_from_sint64(internal_id: i64) -> String {
    let mut out = String::with_capacity(32);
    for byte in (internal_id as u64).to_le_bytes() {
        out.push(hex_digit(byte & 0x0f));
        out.push(hex_digit((byte >> 4) & 0x0f));
    }
    out.push_str("0800000000000000");
    out
}

/// Convenience bundle. Used to share identity table between T8 (tmp_sprite_asset) and T9 (png_meta).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubSpriteIdentity {
    pub name: String,
    pub file_id: i64,
    pub sprite_guid: String,
}

/// Compute identities for a list of sprite names in given order.
pub fn identities_for_names<I, S>(names: I) -> Vec<SubSpriteIdentity>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    names
        .into_iter()
        .map(|name| {
            let name = name.as_ref();
            let file_id = unity_sprite_internal_id(name);
            SubSpriteIdentity {
                name: name.to_string(),
                file_id,
                sprite_guid: unity_guid_from_sint64(file_id),
            }
        })
        .collect()
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => panic!("hex digit out of range: {value}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_id_matches_fixture_for_launcher_aa_lv1() {
        assert_eq!(unity_sprite_internal_id("Launcher_AA_lv1"), -7785096316908003433);
    }

    #[test]
    fn internal_id_matches_fixture_for_launcher_ab_lv5() {
        assert_eq!(unity_sprite_internal_id("Launcher_AB_lv5"), 7109720716409640961);
    }

    #[test]
    fn guid_matches_fixture_for_negative_internal_id() {
        assert_eq!(
            unity_guid_from_sint64(-7785096316908003433),
            "79f1f70a528c5f390800000000000000"
        );
    }

    #[test]
    fn guid_matches_fixture_for_positive_internal_id() {
        assert_eq!(
            unity_guid_from_sint64(7109720716409640961),
            "10815d9925dcaa260800000000000000"
        );
    }

    #[test]
    fn identities_cover_fixture_names() {
        let identities = identities_for_names(FIXTURE_SPRITE_NAMES);
        assert_eq!(identities.len(), 13);
        assert!(identities.iter().all(|identity| {
            identity.sprite_guid.len() == 32
                && identity
                    .sprite_guid
                    .chars()
                    .all(|c| matches!(c, '0'..='9' | 'a'..='f'))
        }));
    }
}
