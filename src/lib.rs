pub mod font;
mod utils;

use std::cell::RefCell;
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
struct Config {
    column_count: u32,
    column_width: u32,
    char_spacing: u32,
    line_spacing: u32,
    column_spacing: u32,
    scaling: u32,
    margins: [u32; 4],
    fg_color: (u8, u8, u8, u8),
    bg_color: (u8, u8, u8, u8),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            column_count: 1,
            column_width: 80,
            char_spacing: 1,
            line_spacing: 1,
            column_spacing: 4,
            scaling: 1,
            margins: [0; 4], // top, right, bottom, left
            fg_color: (0, 0, 0, 255),
            bg_color: (255, 255, 255, 0),
        }
    }
}

thread_local! {
    static CONFIG: RefCell<Config> = RefCell::new(Config::default());
}

fn format_text(text: &[u8]) -> (usize, Vec<u8>) {
    let mut formatted = Vec::with_capacity(text.len());
    let line_length = CONFIG.with(|c| c.borrow().column_width) as usize;
    let mut pos: usize = 0;
    let mut i: usize = 0;
    let mut line_count: usize = 0;
    while i < text.len() {
        match text[i] {
            b'\n' => {
                formatted.push(b'\n');
                pos = 0;
                line_count += 1;
                i += 1;
            }
            b' ' => {
                if pos < line_length {
                    formatted.push(b' ');
                    pos += 1;
                }
                i += 1;
            }
            _ => {
                let mut wlen = 0usize;
                while i + wlen < text.len() {
                    let b = text[i + wlen];
                    if b == b' ' || b == b'\n' {
                        break;
                    }
                    wlen += 1;
                }

                if pos + wlen <= line_length {
                    formatted.extend(text[i..i + wlen].iter().map(|c| c.to_ascii_uppercase()));
                    i += wlen;
                    pos += wlen;
                } else {
                    let mut remaining = wlen;
                    while remaining > 0 {
                        let mut rem_space = line_length.saturating_sub(pos);
                        if rem_space == 0 {
                            formatted.push(b'\n');
                            pos = 0;
                            line_count += 1;
                            rem_space = line_length;
                        }

                        if remaining + pos <= line_length {
                            formatted.extend(
                                text[i..i + remaining]
                                    .iter()
                                    .map(|c| c.to_ascii_uppercase()),
                            );
                            pos += remaining;
                            i += remaining;
                            remaining = 0;
                        } else {
                            if rem_space == 1 {
                                formatted.push(b'\n');
                                pos = 0;
                                line_count += 1;
                                continue;
                            }
                            let take = rem_space - 1;
                            formatted
                                .extend(text[i..i + take].iter().map(|c| c.to_ascii_uppercase()));
                            formatted.push(b'\n');
                            line_count += 1;
                            pos = 0;
                            i += take;
                            remaining -= take;
                        }
                    }
                }
            }
        }
    }
    while formatted
        .last()
        .copied()
        .is_some_and(|c| c.is_ascii_whitespace())
    {
        formatted.pop();
        line_count = line_count.saturating_sub(1);
    }
    (line_count + 1, formatted)
}

fn render_pixels(total_lines: usize, text: &[u8]) -> Vec<Vec<bool>> {
    let cfg = CONFIG.with(|c| c.borrow().clone());
    let line_per_column = (total_lines as u32).div_ceil(cfg.column_count);
    let width = if total_lines == 1 {
        text.len() as u32 * (font::CHAR_WIDTH + cfg.char_spacing)
            - cfg.char_spacing
    } else {
        cfg.column_count
            * (cfg.column_width * (font::CHAR_WIDTH + cfg.char_spacing) - cfg.char_spacing
                + cfg.column_spacing)
            - cfg.column_spacing
    } + cfg.margins[1]
        + cfg.margins[3];
    let height = line_per_column * (font::CHAR_HEIGHT + cfg.line_spacing) - cfg.line_spacing
        + cfg.margins[0]
        + cfg.margins[2];
    let mut pixels = vec![vec![false; width as usize]; height as usize];
    for (line_idx, line) in text.split(|&c| c == b'\n').enumerate() {
        let col_idx = (line_idx as u32) / line_per_column;
        let row_idx = (line_idx as u32) % line_per_column;
        let x_start = cfg.margins[3]
            + col_idx
                * (cfg.column_width * (font::CHAR_WIDTH + cfg.char_spacing) - cfg.char_spacing
                    + cfg.column_spacing);
        let y_start = cfg.margins[0] + row_idx * (font::CHAR_HEIGHT + cfg.line_spacing);
        let mut x = x_start;
        for &c in line {
            if c == b' ' {
                x += font::CHAR_WIDTH + cfg.char_spacing;
                continue;
            }
            let bitmap = font::get_char_bitmap(c);
            for sy in 0..font::CHAR_HEIGHT {
                for sx in 0..font::CHAR_WIDTH {
                    if font::get_pixel(bitmap, sx, sy) {
                        let px = x + sx;
                        let py = y_start + sy;
                        pixels[py as usize][px as usize] = true;
                    }
                }
            }
            x += font::CHAR_WIDTH + cfg.char_spacing;
        }
    }
    pixels
}

fn render_image(pixels: &[Vec<bool>]) -> Vec<u8> {
    let cfg = CONFIG.with(|c| c.borrow().clone());
    let mut buf: Vec<u8> = Vec::new();
    let mut encoder = png::Encoder::new(
        &mut buf,
        pixels[0].len() as u32 * cfg.scaling,
        pixels.len() as u32 * cfg.scaling,
    );
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    let palette = [
        cfg.bg_color.0,
        cfg.bg_color.1,
        cfg.bg_color.2,
        cfg.fg_color.0,
        cfg.fg_color.1,
        cfg.fg_color.2,
    ];
    encoder.set_palette(&palette);
    let alpha = [cfg.bg_color.3, cfg.fg_color.3];
    encoder.set_trns(&alpha);
    let mut writer = encoder.write_header().unwrap();
    let mut image_data: Vec<u8> = Vec::with_capacity(
        pixels.len() * pixels[0].len() * cfg.scaling as usize * cfg.scaling as usize,
    );
    for row in pixels {
        let pos = image_data.len();
        for &p in row {
            let v = if p { 1u8 } else { 0u8 };
            for _ in 0..cfg.scaling {
                image_data.push(v);
            }
        }
        for _ in 1..cfg.scaling {
            image_data.extend_from_within(pos..pos + pixels[0].len() * cfg.scaling as usize)
        }
    }
    writer.write_image_data(&image_data).unwrap();
    writer.finish().unwrap();
    buf
}

#[wasm_bindgen]
pub fn set_config(config_str: &str) -> Result<(), JsValue> {
    let json = serde_json::from_str::<serde_json::Value>(config_str)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {}", e)))?;
    if !json.is_object() {
        return Err(JsValue::from_str("Config must be a JSON object"));
    }
    let obj = json.as_object().unwrap();
    CONFIG.with(|c| {
        let mut config = c.borrow_mut();
        for (k, v) in obj {
            match (k.as_str(), v) {
                ("column_count", serde_json::Value::Number(n)) => {
                    if let Some(n) = n.as_u64() {
                        config.column_count = n as u32;
                    }
                }
                ("column_width", serde_json::Value::Number(n)) => {
                    if let Some(n) = n.as_u64() {
                        config.column_width = n as u32;
                    }
                }
                ("char_spacing", serde_json::Value::Number(n)) => {
                    if let Some(n) = n.as_u64() {
                        config.char_spacing = n as u32;
                    }
                }
                ("line_spacing", serde_json::Value::Number(n)) => {
                    if let Some(n) = n.as_u64() {
                        config.line_spacing = n as u32;
                    }
                }
                ("column_spacing", serde_json::Value::Number(n)) => {
                    if let Some(n) = n.as_u64() {
                        config.column_spacing = n as u32;
                    }
                }
                ("scaling", serde_json::Value::Number(n)) => {
                    if let Some(n) = n.as_u64() {
                        config.scaling = n as u32;
                    }
                }
                ("margins", serde_json::Value::Object(o)) => {
                    if let Some(t) = o.get("top").and_then(|v| v.as_u64()) {
                        config.margins[0] = t as u32;
                    }
                    if let Some(r) = o.get("right").and_then(|v| v.as_u64()) {
                        config.margins[1] = r as u32;
                    }
                    if let Some(b) = o.get("bottom").and_then(|v| v.as_u64()) {
                        config.margins[2] = b as u32;
                    }
                    if let Some(l) = o.get("left").and_then(|v| v.as_u64()) {
                        config.margins[3] = l as u32;
                    }
                }
                ("fg_color", serde_json::Value::Array(arr)) => {
                    if arr.len() == 4
                        && arr.iter().all(|v| {
                            v.is_number() && v.as_u64().is_some() && v.as_u64().unwrap() <= 255
                        })
                    {
                        config.fg_color = (
                            arr[0].as_u64().unwrap() as u8,
                            arr[1].as_u64().unwrap() as u8,
                            arr[2].as_u64().unwrap() as u8,
                            arr[3].as_u64().unwrap() as u8,
                        );
                    }
                }
                ("bg_color", serde_json::Value::Array(arr)) => {
                    if arr.len() == 4
                        && arr.iter().all(|v| {
                            v.is_number() && v.as_u64().is_some() && v.as_u64().unwrap() <= 255
                        })
                    {
                        config.bg_color = (
                            arr[0].as_u64().unwrap() as u8,
                            arr[1].as_u64().unwrap() as u8,
                            arr[2].as_u64().unwrap() as u8,
                            arr[3].as_u64().unwrap() as u8,
                        );
                    }
                }
                _ => return Err(JsValue::from_str(&format!("Invalid config: {}", k))),
            }
        }
        Ok(())
    })
}

#[wasm_bindgen]
pub fn set_panic_hook() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub fn create_image(text: &[u8]) -> Vec<u8> {
    let (line_count, formatted_text) = format_text(text);
    let pixels = render_pixels(line_count, &formatted_text);
    render_image(&pixels)
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}
