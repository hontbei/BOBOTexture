// AtlasPacker Pro - full-feature TexturePacker replica with TextMeshPro Sprite Asset export.
//
// Module layout (filled in across Waves 2-5):
//   - model:       request/response data structures        [Wave 2]
//   - preprocess:  image trim / extrude / sub-rect crop    [Wave 2]
//   - packer:      Skyline / MaxRects / Guillotine         [Wave 2 (Skyline) / Wave 3]
//   - unity_meta:  Unity .meta sprite-sheet parser/writer  [Wave 2 (parser) / Wave 4 (writer)]
//   - compositor:  atlas image composition + variants      [Wave 3]
//   - exporters:   JSON-Array, TMP Sprite Asset, Cocos ... [Wave 4]

pub mod model;
pub mod preprocess;
pub mod packer;
pub mod compositor;
pub mod resolution_variants;
pub mod exporters;
pub mod scanner;
pub mod pipeline;
pub mod sub_sprite_id;
pub mod unity_meta;
