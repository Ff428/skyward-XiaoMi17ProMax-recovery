//! 主题颜色与壁纸：主色 / 页面底色 / 磁贴底色，以及那张自己挑的图
//! （2026-11-28，取自 skyward-pudding v0.5.14）。

use super::*;
use crate::theme::palette_of;
use crate::wallpaper::Wallpaper;
use std::path::{Path, PathBuf};

/// 主色：主人挑过就用它，否则用当前主题自带的那一个。
pub(crate) fn accent_of(id: u8, accent: Option<(u8, u8, u8)>) -> (u8, u8, u8) {
    accent.unwrap_or_else(|| {
        let p = palette_of(id);
        (p.accent.r, p.accent.g, p.accent.b)
    })
}

/// 页面底色：同上。
pub(crate) fn background_of(id: u8, background: Option<(u8, u8, u8)>) -> (u8, u8, u8) {
    background.unwrap_or_else(|| {
        let p = palette_of(id);
        (p.bg.r, p.bg.g, p.bg.b)
    })
}

/// 磁贴底色：只管关着的那些磁贴，默认取主题的卡片色。
pub(crate) fn tile_of(id: u8, tile: Option<(u8, u8, u8)>) -> (u8, u8, u8) {
    tile.unwrap_or_else(|| {
        let p = palette_of(id);
        (p.card.r, p.card.g, p.card.b)
    })
}

impl App {
    /// 换一套主题：0 亮、1 暗、2 柔光、3 暗光。
    pub fn set_theme(&mut self, id: u8) {
        self.state.theme = id.min(3);
        self.state.dark = matches!(self.state.theme, 1 | 3);
        self.refresh();
    }

    /// 把壁纸文件夹里的一张选上；`None` 就是去掉壁纸。
    pub fn pick_wallpaper(&mut self, file: Option<PathBuf>) {
        self.state.wallpaper_file = file.map(|p| p.display().to_string());
        self.load_wallpaper();
        self.refresh();
    }

    /// 按设置里的路径重新读一遍壁纸（启动时读一次就够）。
    pub fn load_wallpaper(&mut self) {
        let Some(path) = self.state.wallpaper_file.clone() else {
            self.state.wallpaper_paper = None;
            return;
        };
        if !Path::new(&path).is_file() {
            self.state.wallpaper_file = None;
            self.state.wallpaper_paper = None;
            return;
        }
        let (w, h) = (self.w, self.h);
        let s = &self.state;
        self.state.wallpaper_paper =
            Wallpaper::read_placed(Path::new(&path), w, h, s.wallpaper_zoom, s.wallpaper_pan_x, s.wallpaper_pan_y).ok();
    }

    /// 同一张图，按新的缩放与位置重采样：不碰磁盘。
    pub fn reshape_wallpaper(&mut self) {
        let (zoom, px, py) = (self.state.wallpaper_zoom, self.state.wallpaper_pan_x, self.state.wallpaper_pan_y);
        if let Some(paper) = &self.state.wallpaper_paper {
            self.state.wallpaper_paper = Some(paper.placed(zoom, px, py));
        }
        self.refresh();
    }

    /// 生效的壁纸文件夹：设置里那个，读不到就回落到出厂的 /sdcard/Skyward
    /// （老设置文件里写的是模拟存储的路径，recovery 里没有那个挂载）。
    pub fn wallpaper_folder(&self) -> String {
        let dir = self.state.wallpaper_dir.trim().to_string();
        if dir.is_empty() || dir == crate::wallpaper::OLD_DIR || !Path::new(&dir).is_dir() {
            return crate::wallpaper::DEFAULT_DIR.to_string();
        }
        dir
    }

    /// 读壁纸文件夹里的图，顺手做一遍缩略图（挑图那一页用）。
    pub fn list_wallpapers(&mut self) {
        let dir = self.wallpaper_folder();
        self.state.wallpaper_dir = dir.clone();
        let listed = crate::wallpaper::list(Path::new(&dir));
        self.state.wallpaper_thumbs = match &listed {
            Ok(files) => files.iter().take(24).filter_map(|p| Some((p.display().to_string(), Wallpaper::read(p, 220, 220).ok()?))).collect(),
            Err(_) => Vec::new(),
        };
        self.state.wallpaper_files = Some(listed.map_err(|e| format!("{e:#}")));
        self.refresh();
    }

    /// 壁纸这一帧要贴的那一层：压暗 + 薄纱，按三个值缓存，变了才重算。
    pub(super) fn wallpaper_pixels(&self, t: &Theme) -> Option<std::sync::Arc<Vec<u8>>> {
        let s = &self.state;
        let paper = s.wallpaper_paper.as_ref()?;
        let key = (s.wallpaper_dim * 255.0).round() as u64
            | ((s.foreground * 255.0).round() as u64) << 8
            | u64::from(t.veil.r) << 16
            | u64::from(t.veil.g) << 24
            | u64::from(t.veil.b) << 32
            | u64::from(t.veil.a) << 40;
        let mut guard = paper.softened.lock().ok()?;
        if let Some((k, px)) = guard.as_ref()
            && *k == key
        {
            return Some(px.clone());
        }
        let veil = crate::color::Rgba::rgb(t.veil.r, t.veil.g, t.veil.b).with_alpha(t.veil.a);
        let px = std::sync::Arc::new(crate::wallpaper::dimmed_veiled(&paper.rgba, s.wallpaper_dim, veil));
        *guard = Some((key, px.clone()));
        Some(px)
    }
}
