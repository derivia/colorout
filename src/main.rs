use clap::Parser;
use image::{DynamicImage, GenericImageView, Rgba};
use std::collections::HashMap;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(help = "Image to get color palette")]
    image_path: String,

    #[arg(long, default_value = "5", help = "Color quantity in palette", value_parser = clap::value_parser!(u8).range(1..=25))]
    quant: u8,
}

const COLOR_RANGE_INTERVAL: u8 = 16;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct ColorRange {
    lower: Rgba<u8>,
    upper: Rgba<u8>,
    representative: Rgba<u16>,
}

fn generate_color_ranges(step: u8) -> Vec<ColorRange> {
    let validated_step = step.clamp(1, 64);
    let mut ranges = Vec::with_capacity(256 * 256 * 256 / (step as usize * step as usize));

    for r in (0u8..=255).step_by(validated_step as usize) {
        for g in (0u8..=255).step_by(validated_step as usize) {
            for b in (0u8..=255).step_by(validated_step as usize) {
                let lower = Rgba([r, g, b, 255]);
                let upper = Rgba([
                    r.saturating_add(validated_step.saturating_sub(1)),
                    g.saturating_add(validated_step.saturating_sub(1)),
                    b.saturating_add(validated_step.saturating_sub(1)),
                    255,
                ]);
                let representative = Rgba([
                    (r as u16 + upper.0[0] as u16) / 2,
                    (g as u16 + upper.0[1] as u16) / 2,
                    (b as u16 + upper.0[2] as u16) / 2,
                    255,
                ]);
                ranges.push(ColorRange {
                    lower,
                    upper,
                    representative,
                });
            }
        }
    }
    ranges
}

fn is_in_range(color: &Rgba<u8>, range: &ColorRange) -> bool {
    let Rgba([r, g, b, _]) = color;
    let Rgba([r_min, g_min, b_min, _]) = range.lower;
    let Rgba([r_max, g_max, b_max, _]) = range.upper;
    r >= &r_min && r <= &r_max && g >= &g_min && g <= &g_max && b >= &b_min && b <= &b_max
}

fn find_representative_color(color: &Rgba<u8>, ranges: &[ColorRange]) -> Option<ColorRange> {
    ranges
        .iter()
        .find(|range| is_in_range(color, range))
        .cloned()
}

fn rgba_to_hex(color: Rgba<u16>) -> String {
    format!("#{:02X}{:02X}{:02X}", color.0[0], color.0[1], color.0[2])
}

fn img_to_palette(img: &DynamicImage, top_n: u8) -> Vec<String> {
    let palette_ranges = generate_color_ranges(COLOR_RANGE_INTERVAL);
    let mut color_counts: HashMap<ColorRange, usize> = HashMap::new();

    let stride = 3;
    for y in (0..img.height()).step_by(stride as usize) {
        for x in (0..img.width()).step_by(stride as usize) {
            let pixel = img.get_pixel(x, y);
            if let Some(representative_range) = find_representative_color(&pixel, &palette_ranges) {
                *color_counts.entry(representative_range).or_insert(0) += 1;
            }
        }
    }

    let mut sorted_colors: Vec<_> = color_counts.into_iter().collect();
    sorted_colors.sort_by(|a, b| b.1.cmp(&a.1));

    sorted_colors
        .into_iter()
        .take(top_n as usize)
        .map(|(color_range, _count)| rgba_to_hex(color_range.representative))
        .collect()
}

fn main() -> Result<(), image::ImageError> {
    let args = Cli::parse();
    let img = image::open(&args.image_path)?;
    let quant = &args.quant;

    let color_palette = img_to_palette(&img, *quant);
    println!("{:?}", color_palette);

    Ok(())
}
