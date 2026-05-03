use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Document representation of a Unity `.asset.meta` file using the
/// `NativeFormatImporter` importer type (used for TMP SpriteAsset objects).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetMetaDoc {
    pub file_format_version: u32,
    pub guid: String,
    pub main_object_file_id: i64,
    pub user_data: String,
    pub asset_bundle_name: String,
    pub asset_bundle_variant: String,
    pub external_objects: BTreeMap<String, String>,
}

/// Render an `AssetMetaDoc` to a UTF-8 string (LF only, no BOM).
/// Output matches the frozen fixture byte-for-byte when given the same inputs.
pub fn render_asset_meta_doc(doc: &AssetMetaDoc) -> String {
    let mut out = String::with_capacity(256);
    writeln!(&mut out, "fileFormatVersion: {}", doc.file_format_version).unwrap();
    writeln!(&mut out, "guid: {}", doc.guid).unwrap();
    out.push_str("NativeFormatImporter:\n");

    if doc.external_objects.is_empty() {
        out.push_str("  externalObjects: {}\n");
    } else {
        out.push_str("  externalObjects:\n");
        for (key, value) in &doc.external_objects {
            writeln!(&mut out, "    {}: {}", key, value).unwrap();
        }
    }

    writeln!(&mut out, "  mainObjectFileID: {}", doc.main_object_file_id).unwrap();

    // Trailing-space lines: colon + space + LF (intentional for byte-exact match)
    out.push_str("  userData: \n");
    out.push_str("  assetBundleName: \n");
    out.push_str("  assetBundleVariant: \n");

    out
}

/// Convenience wrapper: render then convert to `Vec<u8>` (UTF-8, LF only, no BOM).
pub fn emit_asset_meta_doc(doc: &AssetMetaDoc) -> Vec<u8> {
    render_asset_meta_doc(doc).into_bytes()
}

/// Strict parser for the frozen fixture bytes.
/// Panics on any schema mismatch. Asserts `mainObjectFileID == 11400000`.
pub fn doc_from_fixture_bytes(bytes: &[u8]) -> AssetMetaDoc {
    let body = std::str::from_utf8(bytes)
        .unwrap_or_else(|err| panic!("atlas.asset.meta fixture is not utf-8: {err}"));

    let lines: Vec<&str> = body.lines().collect();

    assert!(lines.len() >= 8, "fixture has fewer than 8 lines");

    let file_format_version_line = lines[0];
    assert!(
        file_format_version_line.starts_with("fileFormatVersion: "),
        "first line must be fileFormatVersion"
    );
    let file_format_version: u32 = file_format_version_line["fileFormatVersion: ".len()..]
        .trim()
        .parse()
        .unwrap_or_else(|err| panic!("invalid fileFormatVersion: {err}"));

    let guid_line = lines[1];
    assert!(guid_line.starts_with("guid: "), "second line must be guid");
    let guid = guid_line["guid: ".len()..].trim().to_string();

    assert_eq!(lines[2], "NativeFormatImporter:", "third line must be NativeFormatImporter:");

    assert_eq!(lines[3], "  externalObjects: {}", "line 4 must be '  externalObjects: {{}}'");

    let main_object_line = lines[4];
    assert!(
        main_object_line.starts_with("  mainObjectFileID: "),
        "line 5 must be mainObjectFileID"
    );
    let main_object_file_id: i64 = main_object_line["  mainObjectFileID: ".len()..]
        .trim()
        .parse()
        .unwrap_or_else(|err| panic!("invalid mainObjectFileID: {err}"));

    assert_eq!(
        main_object_file_id, 11400000,
        "mainObjectFileID must be 11400000 (TMP constant)"
    );

    assert_eq!(lines[5], "  userData: ", "line 6 must be '  userData: ' (with trailing space)");
    assert_eq!(lines[6], "  assetBundleName: ", "line 7 must be '  assetBundleName: ' (with trailing space)");
    assert_eq!(lines[7], "  assetBundleVariant: ", "line 8 must be '  assetBundleVariant: ' (with trailing space)");

    AssetMetaDoc {
        file_format_version,
        guid,
        main_object_file_id,
        user_data: String::new(),
        asset_bundle_name: String::new(),
        asset_bundle_variant: String::new(),
        external_objects: BTreeMap::new(),
    }
}

/// Constructor with TMP defaults: file_format_version=2, main_object_file_id=11400000,
/// empty externalObjects, and all string fields empty.
pub fn build_asset_meta_doc(guid: &str) -> AssetMetaDoc {
    AssetMetaDoc {
        file_format_version: 2,
        guid: guid.to_string(),
        main_object_file_id: 11400000,
        user_data: String::new(),
        asset_bundle_name: String::new(),
        asset_bundle_variant: String::new(),
        external_objects: BTreeMap::new(),
    }
}
