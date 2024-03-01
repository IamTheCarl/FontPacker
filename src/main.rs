use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::Write,
    path::Path,
};

use anyhow::{anyhow, Context, Result};
use fontdue::Font;
use image::{imageops, GrayImage, ImageFormat};
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    TargetBin,
};
use serde::Serialize;

fn main() {
    load_font(
        Path::new("testing/Noto_Sans_JP/NotoSansJP-VariableFont_wght.ttf"),
        Path::new("testing/output"),
        64.0,
        2048,
        2048,
    )
    .unwrap();
}

#[derive(Default, Serialize)]
struct FontMeta {
    #[serde(flatten)]
    glyphs: HashMap<char, Glyph>,
}

#[derive(Serialize)]
struct Glyph {
    metrics: GlyphMetrics,
    mapping: GlyphMapping,
}

#[derive(Serialize)]
struct GlyphMapping {
    layer: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct GlyphMetrics {
    xmin: i32,
    ymin: i32,
    width: usize,
    height: usize,
    advance_width: f32,
    advance_height: f32,
}

fn load_font(
    font_path: &Path,
    root_export_path: &Path,
    px: f32,
    layer_width: u32,
    layer_height: u32,
) -> Result<()> {
    let data = fs::read(font_path).context("Failed to read font file into memory")?;
    let font = Font::from_bytes(data, fontdue::FontSettings::default())
        .map_err(|error| anyhow!("Failed to deserialize font: {}", error))?;

    let mut rects_to_place = GroupedRectsToPlace::<_, ()>::new();

    fs::create_dir_all(root_export_path).context("Failed to create output directory")?;

    println!("Loading glyph metrics...");
    for character in font.chars().keys().copied() {
        let metrics = font.metrics(character, px);

        rects_to_place.push_rect(
            character,
            None,
            RectToInsert::new(metrics.width as u32, metrics.height as u32, 1),
        );
    }

    println!("Packing glyphs...");
    let mut target_bins = BTreeMap::new();
    target_bins.insert(0, TargetBin::new(2048, 2048, u32::MAX));

    let rectangle_placements = pack_rects(
        &rects_to_place,
        &mut target_bins,
        &volume_heuristic,
        &contains_smallest_box,
    )
    .map_err(|_| anyhow!("Unable to fit all glyphs into textures. Try a higher resolution."))?;

    let packed_locations = rectangle_placements.packed_locations();

    let mut images = HashMap::new();
    let mut meta_data = FontMeta::default();

    println!("Rastorizing glyphs...");
    for (character, (_container_index, location)) in packed_locations.iter() {
        let layer_index = location.z();

        let layer_image = images
            .entry(layer_index)
            .or_insert_with(|| GrayImage::new(layer_width, layer_height));

        let (metrics, pixels) = font.rasterize(*character, px);

        let glyph_image = GrayImage::from_vec(metrics.width as u32, metrics.height as u32, pixels)
            .expect("Image buffer did not match image size");

        imageops::overlay(
            layer_image,
            &glyph_image,
            location.x() as i64,
            location.y() as i64,
        );

        let layer = layer_file_name(layer_index);
        meta_data.glyphs.insert(
            *character,
            Glyph {
                metrics: GlyphMetrics {
                    xmin: metrics.xmin,
                    ymin: metrics.ymin,
                    width: metrics.width,
                    height: metrics.height,
                    advance_width: metrics.advance_width,
                    advance_height: metrics.advance_height,
                },
                mapping: GlyphMapping {
                    layer,
                    x: location.x(),
                    y: location.y(),
                    width: location.width(),
                    height: location.height(),
                },
            },
        );
    }

    println!("Exporting meta data...");
    let mut meta_file = fs::File::create(root_export_path.join("meta.json"))
        .context("Failed to open `meta.json` for writing.")?;
    serde_json::to_writer_pretty(&mut meta_file, &meta_data)
        .context("Failed to write font meta data.")?;
    meta_file
        .flush()
        .context("Failed to flush data to `meta.json`.")?;

    println!("Exporting image layers...");
    for (index, image) in images {
        let file_path = root_export_path.join(layer_file_name(index));

        if let Err(error) = image.save_with_format(file_path, ImageFormat::Png) {
            println!("Failed to rastorize: {}", error);
        }
    }

    println!("Done.");

    Ok(())
}

fn layer_file_name(index: u32) -> String {
    format!("layer{}.png", index)
}
