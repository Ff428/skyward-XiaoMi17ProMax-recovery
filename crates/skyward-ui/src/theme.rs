use crate::color::{over, Rgba};
use crate::glow::{self, Glow};

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Rgba,
    pub card: Rgba,
    pub text: Rgba,
    pub subtitle: Rgba,
    pub value: Rgba,
    pub section: Rgba,
    pub chevron: Rgba,
    pub accent: Rgba,
    pub track: Rgba,
    pub switch_off: Rgba,
    pub search: Rgba,
    pub search_icon: Rgba,
    pub placeholder: Rgba,
    pub badge: Rgba,
    pub avatar: Rgba,
    pub avatar_glyph: Rgba,
    pub status: Rgba,
    pub battery: BatteryInk,
    pub mono_icon: Rgba,
    pub danger: Rgba,
    pub danger_soft: Rgba,
    /// Titles and values on the parameters page and captions inside cards, a
    /// step softer than `text`.
    pub info: Rgba,
    /// Line icons on the parameters page.
    pub outline: Rgba,
    /// Captions under the About page's spec values.
    pub spec_label: Rgba,
    /// Laid over the card behind a row a finger holds down, under the
    /// row's words and icon.
    pub row_press: Rgba,
    /// Laid over the `<`, the search page's round button and the home
    /// page's search bar while a finger holds them down.
    pub button_press: Rgba,
    /// The round button at the end of a WLAN network's row, and the arrow
    /// in it.
    pub round: Rgba,
    pub round_mark: Rgba,
    /// The arrows up and down after the value of a row that opens a list of
    /// choices.
    pub updown: Rgba,
    /// A list of choices open over the page, and the shadow it casts where
    /// the shadow is whole.
    pub popup: Rgba,
    pub shadow: Rgba,
}

/// The status bar battery's inks, partly see-through as the stock status
/// bar's are, so they sit on whatever page is under them.
#[derive(Clone, Copy)]
pub struct BatteryInk {
    /// What the level fills while the cable is in, charging or full.
    pub charging: Rgba,
    /// What a low level fills with the cable out.
    pub low: Rgba,
    /// What any other level fills; the digits are cut out of it.
    pub level: Rgba,
    /// The empty part, cap included.
    pub empty: Rgba,
    pub bolt: Rgba,
}

/// The About page in a theme: the moving field behind it, the version
/// line under the wordmark, and the cards that scroll over the field.
#[derive(Clone, Copy)]
pub struct Hero {
    pub glow: &'static Glow,
    /// The version line: black or white, partly see-through.
    pub version: Rgba,
    pub card: Rgba,
    pub title: Rgba,
    pub value: Rgba,
    pub chevron: Rgba,
}

/// Layout metrics measured on a Xiaomi 17 Ultra's stock settings, in dp,
/// converted at the panel's density.
#[derive(Clone, Copy)]
pub struct Metrics {
    pub w: u32,
    pub h: u32,
    pub px_per_dp: f32,
    pub margin: f32,
    pub card_radius: f32,
    pub card_gap: f32,
    pub section_block: f32,
    pub section_label_top: f32,
    pub row_h: f32,
    pub row_pad_v: f32,
    pub inner: f32,
    pub icon: f32,
    pub icon_gap: f32,
    pub title_px: f32,
    pub title_x: f32,
    pub title_top: f32,
    /// Back chevron: tip x, centre y, width, height and stroke.
    pub back_x: f32,
    pub back_cy: f32,
    pub back_w: f32,
    pub back_h: f32,
    pub back_stroke: f32,
    pub stroke: f32,
    pub content_top: f32,
    pub search_h: f32,
    pub search_text_inset: f32,
    pub after_search: f32,
    pub row_title_px: f32,
    pub sub_px: f32,
    pub sub_gap: f32,
    pub sub_pitch: f32,
    pub label_px: f32,
    pub account_title_px: f32,
    pub account_row_h: f32,
    /// The band of figures under the device card's name: how tall it is, and
    /// the size of the figure each one leads with.
    pub readout_h: f32,
    pub readout_px: f32,
    /// The word under it: smaller, as on the phone's own card, and small
    /// enough that `充电中 · 7.9W` fits one column of four.
    pub readout_sub_px: f32,
    pub avatar: f32,
    pub avatar_inset: f32,
    pub badge_px: f32,
    pub badge_w: f32,
    pub badge_h: f32,
    pub chevron_w: f32,
    pub chevron_h: f32,
    pub chevron_inset: f32,
    pub chevron_stroke: f32,
    pub value_gap: f32,
    pub switch_w: f32,
    pub switch_h: f32,
    pub knob: f32,
    pub slider_h: f32,
    pub slider_knob: f32,
    pub status_x: f32,
    /// Where the battery ends, from the right edge; with the cable in, the
    /// bolt's box ends at `status_bolt_right` instead and the battery moves
    /// left.
    pub status_right: f32,
    pub status_bolt_right: f32,
    pub status_cy: f32,
    pub status_px: f32,
}

impl Metrics {
    pub fn dp(&self, v: f32) -> f32 {
        v * self.px_per_dp
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub pal: Palette,
    pub hero: Hero,
    pub m: Metrics,
    pub dark: bool,
    /// 盖在壁纸上的那一层薄纱。页面上直接画的字（大标题、分区名、主题名）
    /// 压的是它，不是主题自己的页面色，所以它的厚薄由「前景」那根滑杆定。
    /// 没有壁纸时 alpha 是 255，等于就是页面色（2026-11-28）。
    pub veil: Rgba,
}

impl Theme {
    /// `px_per_dp` is the panel density over 160, e.g. 3.0 at 480 dpi.
    pub fn light(w: u32, h: u32, px_per_dp: f32) -> Self {
        Theme { pal: LIGHT, hero: LIGHT_HERO, m: metrics(w, h, px_per_dp), dark: false, veil: LIGHT.bg }
    }

    pub fn dark(w: u32, h: u32, px_per_dp: f32) -> Self {
        Theme { pal: DARK, hero: DARK_HERO, m: metrics(w, h, px_per_dp), dark: true, veil: DARK.bg }
    }

    /// 柔光：淡蓝底、透光白卡、蓝紫点缀（2026-11-28）。
    pub fn glass(w: u32, h: u32, px_per_dp: f32) -> Self {
        Theme { pal: GLASS, hero: GLASS_HERO, m: metrics(w, h, px_per_dp), dark: false, veil: GLASS.bg }
    }

    /// 暗光：柔光的夜间版——近黑的深灰蓝底、透光灰蓝卡（2026-11-28）。
    pub fn dusk(w: u32, h: u32, px_per_dp: f32) -> Self {
        Theme { pal: DUSK, hero: DUSK_HERO, m: metrics(w, h, px_per_dp), dark: true, veil: DUSK.bg }
    }

    /// 按主题号取：0 亮、1 暗、2 柔光、3 暗光。
    pub fn of(id: u8, w: u32, h: u32, px_per_dp: f32) -> Self {
        match id {
            1 => Self::dark(w, h, px_per_dp),
            2 => Self::glass(w, h, px_per_dp),
            3 => Self::dusk(w, h, px_per_dp),
            _ => Self::light(w, h, px_per_dp),
        }
    }

    /// Which of the four themes this is: 0 light, 1 dark, 2 glass, 3 dusk.
    pub fn id(&self) -> u8 {
        if self.pal.accent == DUSK.accent {
            3
        } else if self.dark {
            1
        } else if self.pal.accent == GLASS.accent {
            2
        } else {
            0
        }
    }

    /// 换上主人自己挑的主色与页面底色。底色一换，卡片与文字按它的亮度
    /// 换到深底 / 浅底那一套，两端都不会糊（2026-11-28）。
    pub fn tinted(mut self, accent: Option<(u8, u8, u8)>, background: Option<(u8, u8, u8)>) -> Theme {
        if let Some((r, g, b)) = accent {
            self.pal.accent = Rgba::rgb(r, g, b);
        }
        if let Some((r, g, b)) = background {
            self.pal = with_background(self.pal, Rgba::rgb(r, g, b));
            self.dark = luminance(self.pal.bg) < 140.0;
        }
        self
    }

    /// 薄纱的厚薄：`foreground` 是 0..1，1 全不透明（页面色），0 全透
    /// （只剩壁纸）。没有壁纸时页面色的 alpha 本来就是 255，涂了也看不见
    /// （2026-11-28）。
    pub fn veiled(mut self, foreground: f32, has_paper: bool) -> Theme {
        let base = self.pal.bg;
        self.veil = if has_paper {
            base.with_alpha((foreground.clamp(0.0, 1.0) * 255.0).round() as u8)
        } else {
            base
        };
        self
    }

    /// This theme with every card as it shows behind a row held down.
    pub fn pressed(&self) -> Theme {
        let mut t = *self;
        t.pal.card = over(self.pal.row_press, self.pal.card);
        t.hero.card = over(self.pal.row_press, self.hero.card);
        t
    }
}

/// 0..255 的亮度，用来判断一个底色该配深字还是浅字。
fn luminance(c: Rgba) -> f32 {
    0.2126 * f32::from(c.r) + 0.7152 * f32::from(c.g) + 0.0722 * f32::from(c.b)
}

/// 某个主题号的调色板（不用尺寸）：给「还没改过」的颜色取初值用。
pub fn palette_of(id: u8) -> Palette {
    match id {
        1 => DARK,
        2 => GLASS,
        3 => DUSK,
        _ => LIGHT,
    }
}

/// 换页面底色：卡片、文字、箭头、状态栏都跟着底色的明暗翻一档。
fn with_background(mut pal: Palette, bg: Rgba) -> Palette {
    let dark = luminance(bg) < 140.0;
    pal.bg = bg;
    if dark {
        pal.card = Rgba::rgb(30, 30, 34);
        pal.text = Rgba::rgb(238, 238, 244);
        pal.subtitle = Rgba::rgb(152, 154, 166);
        pal.value = Rgba::rgb(120, 122, 136);
        pal.section = Rgba::rgb(150, 156, 184);
        pal.chevron = Rgba::rgb(96, 98, 116);
        pal.search = Rgba::rgb(38, 38, 44);
        pal.search_icon = Rgba::rgb(140, 142, 158);
        pal.placeholder = Rgba::rgb(230, 232, 244).with_alpha(84);
        pal.status = Rgba::rgb(230, 232, 244);
        pal.info = Rgba::rgb(210, 212, 224);
        pal.outline = Rgba::rgb(160, 164, 184);
        pal.spec_label = Rgba::rgb(130, 134, 156);
        pal.round = Rgba::rgb(44, 44, 52);
        pal.round_mark = Rgba::rgb(140, 142, 160);
        pal.updown = Rgba::rgb(70, 72, 92);
        pal.popup = Rgba::rgb(42, 42, 50);
        pal.mono_icon = Rgba::rgb(130, 134, 156);
        pal.avatar = Rgba::rgb(60, 62, 76);
        pal.avatar_glyph = Rgba::rgb(190, 194, 214);
        pal.track = Rgba::rgb(46, 46, 56);
        pal.switch_off = Rgba::rgb(60, 62, 74);
        pal.badge = Rgba::rgb(40, 44, 62);
        pal.danger_soft = Rgba::rgb(58, 32, 40);
        pal.battery.level = Rgba::rgb(230, 234, 246).with_alpha(226);
        pal.battery.empty = Rgba::rgb(230, 234, 246).with_alpha(92);
        pal.battery.bolt = Rgba::rgb(230, 234, 246).with_alpha(200);
    } else {
        pal.card = Rgba::rgb(255, 255, 255);
        pal.text = Rgba::rgb(20, 22, 28);
        pal.subtitle = Rgba::rgb(96, 100, 112);
        pal.value = Rgba::rgb(140, 146, 162);
        pal.section = Rgba::rgb(108, 116, 140);
        pal.chevron = Rgba::rgb(158, 164, 182);
        pal.search = Rgba::rgb(232, 236, 244);
        pal.search_icon = Rgba::rgb(118, 124, 142);
        pal.placeholder = Rgba::rgb(30, 36, 52).with_alpha(84);
        pal.status = Rgba::rgb(40, 44, 56);
        pal.info = Rgba::rgb(60, 64, 78);
        pal.outline = Rgba::rgb(96, 102, 122);
        pal.spec_label = Rgba::rgb(118, 124, 144);
        pal.round = Rgba::rgb(238, 240, 246);
        pal.round_mark = Rgba::rgb(126, 132, 152);
        pal.updown = Rgba::rgb(186, 192, 210);
        pal.popup = Rgba::rgb(252, 253, 255);
        pal.mono_icon = Rgba::rgb(150, 156, 176);
        pal.avatar = Rgba::rgb(196, 206, 228);
        pal.avatar_glyph = Rgba::rgb(255, 255, 255);
        pal.track = Rgba::rgb(214, 218, 228);
        pal.switch_off = Rgba::rgb(198, 202, 214);
        pal.badge = Rgba::rgb(226, 234, 250);
        pal.danger_soft = Rgba::rgb(250, 228, 228);
        pal.battery.level = Rgba::rgb(24, 38, 70).with_alpha(206);
        pal.battery.empty = Rgba::rgb(24, 38, 70).with_alpha(84);
        pal.battery.bolt = Rgba::rgb(24, 38, 70).with_alpha(180);
    }
    pal
}

fn metrics(w: u32, h: u32, px_per_dp: f32) -> Metrics {
    let dp = |v: f32| v * px_per_dp;
    Metrics {
        w,
        h,
        px_per_dp,
        margin: dp(12.0),
        card_radius: dp(20.0),
        card_gap: dp(12.0),
        section_block: dp(45.67),
        section_label_top: dp(23.0),
        row_h: dp(55.67),
        row_pad_v: dp(17.0),
        inner: dp(16.0),
        icon: dp(28.0),
        icon_gap: dp(20.67),
        title_px: dp(32.0),
        title_x: dp(28.6),
        title_top: dp(117.7),
        back_x: dp(26.3),
        back_cy: dp(75.7),
        back_w: dp(11.7),
        back_h: dp(21.0),
        back_stroke: dp(1.8),
        stroke: dp(2.0),
        content_top: dp(167.7),
        search_h: dp(43.7),
        search_text_inset: dp(43.7),
        after_search: dp(12.7),
        row_title_px: dp(16.9),
        sub_px: dp(14.0),
        sub_gap: dp(6.7),
        sub_pitch: dp(18.7),
        label_px: dp(13.2),
        account_title_px: dp(17.1),
        account_row_h: dp(69.7),
        readout_h: dp(57.3),
        readout_px: dp(17.0),
        readout_sub_px: dp(12.6),
        avatar: dp(34.0),
        avatar_inset: dp(12.8),
        badge_px: dp(14.5),
        badge_w: dp(72.9),
        badge_h: dp(28.9),
        chevron_w: dp(7.7),
        chevron_h: dp(13.3),
        chevron_inset: dp(17.3),
        chevron_stroke: dp(1.15),
        value_gap: dp(11.3),
        switch_w: dp(48.7),
        switch_h: dp(28.0),
        knob: dp(19.3),
        slider_h: dp(27.5),
        slider_knob: dp(18.7),
        status_x: dp(37.0),
        status_right: dp(40.0),
        status_bolt_right: dp(36.0),
        status_cy: dp(25.7),
        status_px: dp(14.1),
    }
}

// Sampled from the stock Settings on a Xiaomi 17 Ultra.
const LIGHT: Palette = Palette {
    bg: Rgba::rgb(243, 243, 243),
    card: Rgba::rgb(255, 255, 255),
    text: Rgba::rgb(0, 0, 0),
    subtitle: Rgba::rgb(102, 102, 102),
    value: Rgba::rgb(153, 153, 153),
    section: Rgba::rgb(143, 156, 174),
    chevron: Rgba::rgb(178, 178, 178),
    accent: Rgba::rgb(74, 128, 247),
    track: Rgba::rgb(240, 240, 240),
    switch_off: Rgba::rgb(229, 229, 229),
    search: Rgba::rgb(229, 229, 229),
    search_icon: Rgba::rgb(161, 161, 161),
    // Black at 30%: 160 over the search field, and darker blue where it
    // crosses the cursor.
    placeholder: Rgba::rgb(0, 0, 0).with_alpha(77),
    badge: Rgba::rgb(235, 242, 254),
    avatar: Rgba::rgb(221, 223, 226),
    avatar_glyph: Rgba::rgb(255, 255, 255),
    status: Rgba::rgb(61, 61, 61),
    // The stock status bar's dark icons, sampled over light pages.
    battery: BatteryInk {
        charging: Rgba::rgb(29, 205, 58),
        low: Rgba::rgb(250, 56, 46),
        level: Rgba::rgb(0, 0, 0).with_alpha(196),
        empty: Rgba::rgb(0, 0, 0).with_alpha(77),
        bolt: Rgba::rgb(0, 0, 0).with_alpha(172),
    },
    mono_icon: Rgba::rgb(179, 179, 179),
    danger: Rgba::rgb(230, 66, 60),
    danger_soft: Rgba::rgb(253, 236, 235),
    info: Rgba::rgb(51, 51, 51),
    outline: Rgba::rgb(102, 102, 102),
    spec_label: Rgba::rgb(127, 127, 127),
    // A white card held down goes to 225; the page under `<` goes from 243
    // to 219, the search field and its round button from 229 to 206.
    row_press: Rgba::rgb(0, 0, 0).with_alpha(30),
    button_press: Rgba::rgb(0, 0, 0).with_alpha(26),
    // Sampled from the stock WLAN page.
    round: Rgba::rgb(242, 242, 242),
    round_mark: Rgba::rgb(169, 169, 169),
    // Sampled from the stock page that adds a network; the shadow as the
    // grey it leaves on the page beside the list is fitted.
    updown: Rgba::rgb(214, 214, 214),
    popup: Rgba::rgb(255, 255, 255),
    shadow: Rgba::rgb(0, 0, 0).with_alpha(27),
};

const DARK: Palette = Palette {
    bg: Rgba::rgb(0, 0, 0),
    card: Rgba::rgb(28, 28, 28),
    text: Rgba::rgb(237, 237, 237),
    subtitle: Rgba::rgb(140, 140, 140),
    value: Rgba::rgb(128, 128, 128),
    section: Rgba::rgb(107, 113, 134),
    chevron: Rgba::rgb(77, 77, 77),
    accent: Rgba::rgb(84, 136, 248),
    track: Rgba::rgb(46, 46, 46),
    switch_off: Rgba::rgb(58, 58, 58),
    search: Rgba::rgb(28, 28, 28),
    search_icon: Rgba::rgb(110, 110, 110),
    // White at 36%: 110 over the search field.
    placeholder: Rgba::rgb(255, 255, 255).with_alpha(92),
    badge: Rgba::rgb(29, 40, 64),
    avatar: Rgba::rgb(51, 51, 51),
    avatar_glyph: Rgba::rgb(140, 140, 140),
    status: Rgba::rgb(224, 224, 224),
    // The stock status bar's light icons, sampled over black and over the
    // lock screen's wallpaper.
    battery: BatteryInk {
        // As the stock Settings fill it over black and over a dark grey: the
        // green they fill with over light pages too.
        charging: Rgba::rgb(29, 205, 58),
        low: Rgba::rgb(230, 75, 59),
        level: Rgba::rgb(255, 255, 255).with_alpha(232),
        empty: Rgba::rgb(255, 255, 255).with_alpha(94),
        // As the stock Settings draw it over black and over a dark grey.
        bolt: Rgba::rgb(255, 255, 255).with_alpha(207),
    },
    mono_icon: Rgba::rgb(110, 110, 110),
    danger: Rgba::rgb(255, 92, 84),
    danger_soft: Rgba::rgb(52, 24, 24),
    info: Rgba::rgb(214, 214, 214),
    outline: Rgba::rgb(140, 140, 140),
    spec_label: Rgba::rgb(128, 128, 128),
    // White as strong as the black in the light theme.
    row_press: Rgba::rgb(255, 255, 255).with_alpha(30),
    button_press: Rgba::rgb(255, 255, 255).with_alpha(26),
    // As far from the card as the light theme's are from its own.
    round: Rgba::rgb(41, 41, 41),
    round_mark: Rgba::rgb(118, 118, 118),
    // The arrows as much softer than the chevron as in the light theme; the
    // list a step lighter than the cards it opens over, and its shadow
    // darker, for it to show on them.
    updown: Rgba::rgb(54, 54, 54),
    popup: Rgba::rgb(41, 41, 41),
    shadow: Rgba::rgb(0, 0, 0).with_alpha(77),
};

/// 柔光：淡蓝底、透光白卡、蓝紫点缀（2026-11-28）。
const GLASS: Palette = Palette {
    bg: Rgba::rgb(226, 236, 250),
    card: Rgba::rgb(255, 255, 255).with_alpha(224),
    text: Rgba::rgb(24, 34, 58),
    subtitle: Rgba::rgb(86, 102, 134),
    value: Rgba::rgb(132, 148, 182),
    section: Rgba::rgb(102, 128, 180),
    chevron: Rgba::rgb(150, 169, 205),
    accent: Rgba::rgb(66, 118, 238),
    track: Rgba::rgb(200, 214, 240),
    switch_off: Rgba::rgb(193, 206, 231),
    search: Rgba::rgb(214, 226, 246).with_alpha(230),
    search_icon: Rgba::rgb(118, 139, 181),
    placeholder: Rgba::rgb(24, 40, 74).with_alpha(84),
    badge: Rgba::rgb(218, 232, 254),
    avatar: Rgba::rgb(186, 205, 238),
    avatar_glyph: Rgba::rgb(255, 255, 255),
    status: Rgba::rgb(48, 66, 104),
    battery: BatteryInk {
        charging: Rgba::rgb(29, 205, 58),
        low: Rgba::rgb(230, 75, 59),
        level: Rgba::rgb(24, 38, 70).with_alpha(206),
        empty: Rgba::rgb(24, 38, 70).with_alpha(84),
        bolt: Rgba::rgb(24, 38, 70).with_alpha(180),
    },
    mono_icon: Rgba::rgb(148, 167, 205),
    danger: Rgba::rgb(226, 72, 68),
    danger_soft: Rgba::rgb(250, 226, 226),
    info: Rgba::rgb(48, 68, 110),
    outline: Rgba::rgb(86, 104, 142),
    spec_label: Rgba::rgb(112, 132, 172),
    row_press: Rgba::rgb(46, 90, 180).with_alpha(28),
    button_press: Rgba::rgb(46, 90, 180).with_alpha(24),
    round: Rgba::rgb(226, 236, 252),
    round_mark: Rgba::rgb(128, 149, 190),
    updown: Rgba::rgb(184, 200, 230),
    popup: Rgba::rgb(248, 251, 255).with_alpha(244),
    shadow: Rgba::rgb(24, 48, 96).with_alpha(48),
};

/// 暗光：柔光的夜间版——近黑的深灰蓝底、透光灰蓝卡、蓝紫点缀（2026-11-28）。
const DUSK: Palette = Palette {
    bg: Rgba::rgb(20, 23, 31),
    card: Rgba::rgb(38, 44, 60).with_alpha(226),
    text: Rgba::rgb(232, 236, 246),
    subtitle: Rgba::rgb(152, 160, 182),
    value: Rgba::rgb(116, 126, 152),
    section: Rgba::rgb(102, 114, 148),
    chevron: Rgba::rgb(76, 84, 108),
    accent: Rgba::rgb(112, 152, 242),
    track: Rgba::rgb(42, 48, 64),
    switch_off: Rgba::rgb(56, 62, 82),
    search: Rgba::rgb(38, 44, 60).with_alpha(230),
    search_icon: Rgba::rgb(128, 138, 166),
    placeholder: Rgba::rgb(220, 226, 242).with_alpha(84),
    badge: Rgba::rgb(38, 48, 76),
    avatar: Rgba::rgb(58, 66, 88),
    avatar_glyph: Rgba::rgb(190, 198, 220),
    status: Rgba::rgb(218, 224, 240),
    battery: BatteryInk {
        charging: Rgba::rgb(29, 205, 58),
        low: Rgba::rgb(230, 75, 59),
        level: Rgba::rgb(230, 234, 246).with_alpha(226),
        empty: Rgba::rgb(230, 234, 246).with_alpha(92),
        bolt: Rgba::rgb(230, 234, 246).with_alpha(200),
    },
    mono_icon: Rgba::rgb(120, 130, 160),
    danger: Rgba::rgb(255, 100, 94),
    danger_soft: Rgba::rgb(58, 32, 40),
    info: Rgba::rgb(206, 212, 232),
    outline: Rgba::rgb(150, 160, 190),
    spec_label: Rgba::rgb(124, 134, 166),
    row_press: Rgba::rgb(160, 180, 235).with_alpha(30),
    button_press: Rgba::rgb(160, 180, 235).with_alpha(26),
    round: Rgba::rgb(42, 50, 70),
    round_mark: Rgba::rgb(130, 142, 176),
    updown: Rgba::rgb(60, 70, 96),
    popup: Rgba::rgb(42, 50, 70),
    shadow: Rgba::rgb(0, 0, 0).with_alpha(80),
};

// Sampled from the stock「我的设备」page: in light mode the cards are
// the ordinary settings cards; in dark mode they are white at low opacity
// over the field.
const LIGHT_HERO: Hero = Hero {
    glow: &glow::LIGHT,
    version: Rgba::rgb(0, 0, 0).with_alpha(153),
    card: LIGHT.card,
    title: LIGHT.text,
    value: LIGHT.value,
    chevron: LIGHT.chevron,
};

const DARK_HERO: Hero = Hero {
    glow: &glow::DARK,
    version: Rgba::rgb(255, 255, 255).with_alpha(153),
    card: Rgba::rgb(255, 255, 255).with_alpha(36),
    title: Rgba::rgb(244, 245, 248),
    value: Rgba::rgb(157, 161, 188),
    chevron: Rgba::rgb(118, 122, 160),
};

/// 关于页在柔光主题下的那片流动色场：亮色的光晕，卡片改为淡蓝透光。
const GLASS_HERO: Hero = Hero {
    glow: &glow::GLASS,
    version: Rgba::rgb(24, 34, 58).with_alpha(153),
    card: Rgba::rgb(255, 255, 255).with_alpha(150),
    title: Rgba::rgb(24, 34, 58),
    value: Rgba::rgb(102, 120, 160),
    chevron: Rgba::rgb(140, 160, 198),
};

/// 关于页在暗光主题下的流动色场：冷灰光晕、低透明灰蓝卡。
const DUSK_HERO: Hero = Hero {
    glow: &glow::DUSK,
    version: Rgba::rgb(232, 236, 246).with_alpha(153),
    card: Rgba::rgb(38, 46, 66).with_alpha(150),
    title: Rgba::rgb(232, 236, 246),
    value: Rgba::rgb(152, 162, 190),
    chevron: Rgba::rgb(112, 124, 156),
};
