//! 壁纸：一张自己挑的图，铺在每一页的最底下。压暗与「前景」那层薄纱在
//! 画的时候按滑杆的值合成，原图只解码一次（2026-11-28，取自 skyward-pudding
//! v0.5.14）。
//!
//! 壁纸来自一个文件夹（默认 `/storage/emulated/0/Skyward`），里面放 jpg、png
//! 就能在「挑一张图」里选到；内置的那四张已经删掉。

use crate::color::Rgba;
use anyhow::{ensure, Context, Result};
use std::path::Path;

/// 出厂用的壁纸文件夹。recovery 里 /storage/emulated/0 不挂，/sdcard 才是
/// 数据分区上那个（2026-11-28 真机探到的）。
pub const DEFAULT_DIR: &str = "/sdcard/Skyward";

/// 老设置文件里写过的那个（模拟存储的路径），recovery 里读不到。
pub const OLD_DIR: &str = "/storage/emulated/0/Skyward";

/// 一张已经缩放到屏幕大小的壁纸，四个通道。
#[derive(Clone, Debug)]
pub struct Wallpaper {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
    /// 解码后的原图，留着好按缩放重新采样，不必再读一遍文件。
    pub source: std::sync::Arc<Source>,
    /// 取的是原图的哪一块：0..1，0.5 居中。
    pub pan_x: f32,
    pub pan_y: f32,
    /// 压暗 + 薄纱之后的那一份，连着算它用的三个值一起存：每帧重算是
    /// 三百多万像素，一帧要几百毫秒（2026-11-28 真机）。
    pub softened: std::sync::Arc<std::sync::Mutex<Option<(u64, std::sync::Arc<Vec<u8>>)>>>,
}

impl PartialEq for Wallpaper {
    fn eq(&self, other: &Self) -> bool {
        // 缓存那字段不参与比较：它只是同一张图的另一种形态。
        self.w == other.w && self.h == other.h && self.rgba == other.rgba && self.pan_x == other.pan_x && self.pan_y == other.pan_y
    }
}

/// 一张解码好的原图。
#[derive(Debug, PartialEq, Eq)]
pub struct Source {
    pub w: u32,
    pub h: u32,
    pub channels: usize,
    pub pixels: Vec<u8>,
}

impl Wallpaper {
    /// 读 `path` 的图，缩放着铺满 `w`×`h`（按短边对齐、居中裁切，不变形）。
    pub fn read(path: &Path, w: u32, h: u32) -> Result<Wallpaper> {
        Self::read_zoomed(path, w, h, 1.0)
    }

    /// 读一张图并按 `zoom` 放大（1.0 刚好铺满，越大越往里裁）。
    pub fn read_zoomed(path: &Path, w: u32, h: u32, zoom: f32) -> Result<Wallpaper> {
        Self::read_placed(path, w, h, zoom, 0.5, 0.5)
    }

    /// 读一张图，按 `zoom` 放大、按 `pan_x`/`pan_y` 取哪一块。
    pub fn read_placed(path: &Path, w: u32, h: u32, zoom: f32, pan_x: f32, pan_y: f32) -> Result<Wallpaper> {
        ensure!(w > 0 && h > 0, "a screen of no size");
        let bytes = std::fs::read(path).with_context(|| format!("open {}", path.display()))?;
        let (pixels, pw, ph, channels) = crate::avatar::decode(&bytes, path)?;
        ensure!(pw > 0 && ph > 0, "{} is a picture of no size", path.display());
        let source = std::sync::Arc::new(Source { w: pw, h: ph, channels, pixels });
        let rgba = cover(&source.pixels, pw, ph, channels, w, h, zoom, pan_x, pan_y);
        Ok(Wallpaper { w, h, rgba, source, pan_x, pan_y, softened: Default::default() })
    }

    /// 同一张原图，换一个缩放重新采样：不碰磁盘。
    pub fn placed(&self, zoom: f32, pan_x: f32, pan_y: f32) -> Wallpaper {
        let src = &self.source;
        let rgba = cover(&src.pixels, src.w, src.h, src.channels, self.w, self.h, zoom, pan_x, pan_y);
        Wallpaper { w: self.w, h: self.h, rgba, source: self.source.clone(), pan_x, pan_y, softened: Default::default() }
    }

    /// 同一张原图，换一个缩放与位置，像素不变（缩放/平移滑杆用它）。
    pub fn panned(&self, zoom: f32, pan_x: f32, pan_y: f32) -> Wallpaper {
        self.placed(zoom, pan_x, pan_y)
    }
}

/// 把 `pw`×`ph` 的图铺满 `w`×`h`：按较大的缩放比覆盖，再按 `pan_x`/`pan_y`
/// （0..1，0.5 居中）决定取哪一块，这样边缘的图也能用上。
fn cover(pixels: &[u8], pw: u32, ph: u32, channels: usize, w: u32, h: u32, zoom: f32, pan_x: f32, pan_y: f32) -> Vec<u8> {
    let zoom = zoom.clamp(1.0, 4.0);
    let sx = w as f32 / pw as f32;
    let sy = h as f32 / ph as f32;
    // 缩放越大，取的源区域越小（等于往里裁），画面看起来就是放大。
    let scale = sx.max(sy) * zoom;
    let (sw, sh) = (((w as f32 / scale) as u32).max(1), ((h as f32 / scale) as u32).max(1));
    let room_x = pw.saturating_sub(sw) as f32;
    let room_y = ph.saturating_sub(sh) as f32;
    let ox = (room_x * pan_x.clamp(0.0, 1.0)).round() as u32;
    let oy = (room_y * pan_y.clamp(0.0, 1.0)).round() as u32;
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for yy in 0..h {
        let src_y = (oy + (yy as u64 * sh as u64 / h as u64) as u32).min(ph - 1);
        for xx in 0..w {
            let src_x = (ox + (xx as u64 * sw as u64 / w as u64) as u32).min(pw - 1);
            let at = ((src_y as usize * pw as usize) + src_x as usize) * channels;
            let px = match channels {
                4 => [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]],
                3 => [pixels[at], pixels[at + 1], pixels[at + 2], 255],
                2 => [pixels[at], pixels[at], pixels[at], pixels[at + 1]],
                _ => [pixels[at], pixels[at], pixels[at], 255],
            };
            out.extend_from_slice(&px);
        }
    }
    out
}

/// 壁纸压暗之后盖在最底下的那层。只压暗、不模糊：模糊整屏是三百多万像素
/// 乘两遍盒式卷积，纯 CPU 上一遍就是几百毫秒（2026-11-28 真机卡死后砍掉）。
pub fn dimmed(rgba: &[u8], dim: f32) -> Vec<u8> {
    let mut out = rgba.to_vec();
    let k = (1.0 - dim.clamp(0.0, 0.95)).clamp(0.05, 1.0);
    for px in out.as_chunks_mut::<4>().0 {
        for ch in px.iter_mut().take(3) {
            *ch = ((*ch as f32) * k).round() as u8;
        }
    }
    out
}

/// 壁纸底下那层的颜色：压暗后取平均，给页面当底色兜底。
pub fn average(rgba: &[u8]) -> Rgba {
    if rgba.is_empty() {
        return Rgba::rgb(0, 0, 0);
    }
    let (mut r, mut g, mut b, mut n) = (0u64, 0u64, 0u64, 0u64);
    for px in rgba.as_chunks::<4>().0 {
        r += u64::from(px[0]);
        g += u64::from(px[1]);
        b += u64::from(px[2]);
        n += 1;
    }
    Rgba::rgb((r / n) as u8, (g / n) as u8, (b / n) as u8)
}

/// 压暗之后再把「前景」那层薄纱合进来：整屏都盖上这层，页面上的字才压得住。
/// 合在缓存里做一次，每帧只剩一次 memcpy（2026-11-28 真机：主题颜色页一帧
/// 60ms，铺一层半透明矩形就够呛）。
pub fn dimmed_veiled(rgba: &[u8], dim: f32, veil: Rgba) -> Vec<u8> {
    let mut out = dimmed(rgba, dim);
    let a = veil.a;
    if a == 0 {
        return out;
    }
    let k = u32::from(a);
    let inv = 255 - k;
    let (vr, vg, vb) = (u32::from(veil.r), u32::from(veil.g), u32::from(veil.b));
    for px in out.as_chunks_mut::<4>().0 {
        px[0] = ((vr * k + u32::from(px[0]) * inv + 127) / 255) as u8;
        px[1] = ((vg * k + u32::from(px[1]) * inv + 127) / 255) as u8;
        px[2] = ((vb * k + u32::from(px[2]) * inv + 127) / 255) as u8;
    }
    out
}

/// 壁纸压暗之后的平均色：拿它当页面底色，页面就像那张图暗下去的样子，
/// 卡片压在上面读得清。
pub fn shade_of(paper: &Wallpaper, dim: f32) -> Rgba {
    let avg = average(&paper.rgba);
    let k = (1.0 - dim.clamp(0.0, 0.95)).clamp(0.05, 1.0);
    Rgba::rgb(
        ((avg.r as f32) * k).round() as u8,
        ((avg.g as f32) * k).round() as u8,
        ((avg.b as f32) * k).round() as u8,
    )
}

/// 文件夹里能当壁纸的图：jpg / jpeg / png，按名字排。
pub fn list(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut out: Vec<std::path::PathBuf> = Vec::new();
    let entries = std::fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else { continue };
        let ext = ext.to_ascii_lowercase();
        if matches!(ext.as_str(), "jpg" | "jpeg" | "png") {
            out.push(path);
        }
    }
    out.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一张两色横条图，宽 4 高 2。
    fn stripes() -> (Vec<u8>, u32, u32) {
        let mut px = Vec::new();
        for _y in 0..2 {
            for x in 0..4 {
                let v = if x < 2 { 200 } else { 20 };
                px.extend_from_slice(&[v, v, v, 255]);
            }
        }
        (px, 4, 2)
    }

    #[test]
    fn a_picture_fills_the_screen_by_its_short_side() {
        let (px, w, h) = stripes();
        let out = cover(&px, w, h, 4, 8, 4, 1.0, 0.5, 0.5);
        assert_eq!(out.len(), 8 * 4 * 4, "铺满目标大小");
        let left = out[(1 * 8 + 1) * 4];
        let right = out[(1 * 8 + 6) * 4];
        assert!(left > right, "左边比右边亮：{left} vs {right}");
    }

    #[test]
    fn dimming_darkens_by_the_share_asked() {
        let (px, _, _) = stripes();
        assert_eq!(dimmed(&px, 0.0), px, "不压就是原图");
        let half = dimmed(&px, 0.5);
        assert!(half[0] < px[0] && half[0] > 0, "压一半落在中间：{}", half[0]);
    }

    #[test]
    fn a_pan_picks_which_part_of_the_picture_is_used() {
        let (px, w, h) = stripes();
        let left = cover(&px, w, h, 4, 2, 2, 2.0, 0.0, 0.5);
        let right = cover(&px, w, h, 4, 2, 2, 2.0, 1.0, 0.5);
        assert!(left[0] > right[0], "左边取亮、右边取暗：{} vs {}", left[0], right[0]);
    }

    #[test]
    fn the_average_is_the_colour_behind_a_page() {
        let (px, _, _) = stripes();
        let avg = average(&px);
        assert!(avg.r > 20 && avg.r < 200, "平均落在两色之间：{}", avg.r);
    }

    #[test]
    fn the_veil_lifts_a_dark_picture_towards_the_page_colour() {
        let (px, _, _) = stripes();
        let veiled = dimmed_veiled(&px, 0.0, Rgba::rgb(255, 255, 255).with_alpha(128));
        assert!(veiled[0] > px[0], "盖一层白纱之后更亮");
    }
}
