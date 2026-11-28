//! The settings page and the pages under it: its sliders and switches,
//! the language, the time zone, and how long the screen stays on.

use super::*;
use anyhow::{Context, Result};
use jiff::tz::TimeZone;

/// The settings' sliders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slider {
    Brightness,
    /// How strongly the motor taps.
    Vibration,
    /// How warm the battery may get before the charge is stepped down, in whole
    /// degrees Celsius from [`THERMAL_LIMIT_LEAST`] to [`THERMAL_LIMIT_MOST`].
    ThermalLimit,
    /// 盖在壁纸上的那层薄纱有多实（2026-11-28）。
    Foreground,
    /// 壁纸压暗。
    WallpaperDim,
    /// 壁纸缩放：1.0 起，滑杆整条是 1..4。
    WallpaperZoom,
    /// 壁纸取哪一块。
    WallpaperPanX,
    WallpaperPanY,
    /// 自定义主色的三个分量。
    AccentRed,
    AccentGreen,
    AccentBlue,
    /// 自定义页面底色的三个分量。
    BackRed,
    BackGreen,
    BackBlue,
    /// 磁贴底色的三个分量。
    TileRed,
    TileGreen,
    TileBlue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Switch {
    Vibrate,
    /// Offer internal storage to a computer over USB.
    FileTransfer,
    /// Compress a backup.
    Compress,
    /// Read a backup back once it is written.
    Verify,
    /// Run a zip's install script only if the zip was signed with one of
    /// the phone's update keys.
    Signature,
    /// Write Skyward back into the recovery partitions an install writes
    /// over.
    KeepRecovery,
    /// Keep the screen lock that opened the user's files for the starts that
    /// follow.
    Remember,
    Wlan,
    /// Step the charge down when the battery is past the limit the charge page
    /// keeps.
    ThermalGuard,
    /// A charge step, which is chosen rather than switched: the one already
    /// chosen is not offered as something to tap.
    ChargeStep(ChargeMode),
    /// Open the WLAN as the recovery starts.
    WlanAuto,
    /// Keep the password of the network joined last, beside the network.
    WlanKeep,
    /// Show the password typed rather than a dot for each character.
    ShowPassword,
    /// Keep the time zone the installed system is set to.
    SystemTimeZone,
    /// 首页的分类栏全都摊开（关掉就全折起来）。单独点某一栏只改那一栏，
    /// 而且都记得住（2026-11-28 用户要的）。
    HomeOpen,
}

/// How long the screen stays on with nothing touched, in milliseconds, in
/// the order the page lists them; `None` keeps it on.
pub const SCREEN_OFF: [Option<u64>; 7] = [Some(15_000), Some(30_000), Some(60_000), Some(120_000), Some(300_000), Some(600_000), None];

/// A part of the world a time zone is chosen from, in the order the page
/// over them lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Americas,
    EuropeAfrica,
    Russia,
    Asia,
    Oceania,
}

impl Region {
    /// The parts of the world, in the order the time zone page lists them.
    pub const ALL: [Region; 5] = [Region::Americas, Region::EuropeAfrica, Region::Russia, Region::Asia, Region::Oceania];

    /// The part of the world's name.
    pub fn name(self) -> Text {
        match self {
            Region::Americas => ["美洲", "美洲", "Americas"],
            Region::EuropeAfrica => ["欧洲与非洲", "歐洲與非洲", "Europe and Africa"],
            Region::Russia => ["俄罗斯", "俄羅斯", "Russia"],
            Region::Asia => ["亚洲", "亞洲", "Asia"],
            Region::Oceania => ["大洋洲", "大洋洲", "Oceania"],
        }
    }
}

/// A time zone the clock and the times on the pages can keep.
#[derive(Clone, Copy, Debug)]
pub struct Zone {
    /// Its name.
    pub name: Text,
    /// The cities it is known by.
    pub cities: Text,
    /// Its name in the tz database, whose rules it keeps.
    pub tz: &'static str,
    /// The other names in the tz database whose places keep the same time
    /// under the zone's name: a system set to one of them shows as the zone,
    /// and keeps the rules of the name it is set to.
    pub also: &'static [&'static str],
    /// The part of the world its page lists it under; none for the one zone
    /// that is no place's own.
    pub region: Option<Region>,
}

const fn zone(region: Region, name: Text, cities: Text, tz: &'static str, also: &'static [&'static str]) -> Zone {
    Zone { name, cities, tz, also, region: Some(region) }
}

/// The name of Coordinated Universal Time in the tz database.
pub const UTC: &str = "UTC";

/// The zone the clock keeps until the installed system's is known or
/// another is chosen, by its name in the tz database.
pub const FIRST_ZONE: &str = "Asia/Shanghai";

/// The time zones the clock and the times on the pages can keep, each
/// region's from west to east.
pub static TIME_ZONES: [Zone; 62] = {
    use Region::{Americas, Asia, EuropeAfrica, Oceania, Russia};
    [
    zone(Americas, ["夏威夷时间", "夏威夷時間", "Hawaii Time"], ["檀香山", "檀香山", "Honolulu"], "Pacific/Honolulu", &[]),
    zone(
        Americas,
        ["阿拉斯加时间", "阿拉斯加時間", "Alaska Time"],
        ["安克雷奇", "安克拉治", "Anchorage"],
        "America/Anchorage",
        &["America/Juneau", "America/Sitka", "America/Nome"],
    ),
    zone(Americas, ["美国太平洋时间", "美國太平洋時間", "Pacific Time"], ["洛杉矶", "洛杉磯", "Los Angeles"], "America/Los_Angeles", &["America/Tijuana"]),
    zone(Americas, ["美国山地时间", "美國山區時間", "Mountain Time"], ["丹佛", "丹佛", "Denver"], "America/Denver", &["America/Boise"]),
    zone(Americas, ["美国中部时间", "美國中部時間", "Central Time"], ["芝加哥", "芝加哥", "Chicago"], "America/Chicago", &["America/Menominee"]),
    zone(Americas, ["墨西哥时间", "墨西哥時間", "Mexico Time"], ["墨西哥城", "墨西哥城", "Mexico City"], "America/Mexico_City", &["America/Monterrey", "America/Merida"]),
    zone(Americas, ["美国东部时间", "美國東部時間", "Eastern Time"], ["纽约", "紐約", "New York"], "America/New_York", &["America/Toronto", "America/Detroit"]),
    zone(Americas, ["哥伦比亚时间", "哥倫比亞時間", "Colombia Time"], ["波哥大", "波哥大", "Bogotá"], "America/Bogota", &[]),
    zone(Americas, ["秘鲁时间", "秘魯時間", "Peru Time"], ["利马", "利馬", "Lima"], "America/Lima", &[]),
    zone(Americas, ["委内瑞拉时间", "委內瑞拉時間", "Venezuela Time"], ["加拉加斯", "卡拉卡斯", "Caracas"], "America/Caracas", &[]),
    zone(
        Americas,
        ["亚马逊时间", "亞馬遜時間", "Amazon Time"],
        ["马瑙斯", "瑪瑙斯", "Manaus"],
        "America/Manaus",
        &["America/Porto_Velho", "America/Boa_Vista", "America/Cuiaba", "America/Campo_Grande"],
    ),
    zone(Americas, ["智利时间", "智利時間", "Chile Time"], ["圣地亚哥", "聖地牙哥", "Santiago"], "America/Santiago", &[]),
    zone(
        Americas,
        ["巴西利亚时间", "巴西利亞時間", "Brasília Time"],
        ["圣保罗、里约热内卢", "聖保羅、里約熱內盧", "São Paulo, Rio de Janeiro"],
        "America/Sao_Paulo",
        &["America/Bahia", "America/Fortaleza", "America/Recife", "America/Belem", "America/Maceio", "America/Araguaina", "America/Santarem"],
    ),
    zone(
        Americas,
        ["阿根廷时间", "阿根廷時間", "Argentina Time"],
        ["布宜诺斯艾利斯", "布宜諾斯艾利斯", "Buenos Aires"],
        "America/Argentina/Buenos_Aires",
        &["America/Buenos_Aires", "America/Argentina/Cordoba", "America/Cordoba", "America/Argentina/Mendoza", "America/Mendoza"],
    ),
    zone(
        EuropeAfrica,
        ["英国时间", "英國時間", "British Time"],
        ["伦敦", "倫敦", "London"],
        "Europe/London",
        &["Europe/Belfast", "Europe/Jersey", "Europe/Guernsey", "Europe/Isle_of_Man"],
    ),
    zone(
        EuropeAfrica,
        ["中欧时间", "中歐時間", "Central European Time"],
        ["柏林、巴黎", "柏林、巴黎", "Berlin, Paris"],
        "Europe/Berlin",
        &["Europe/Paris", "Europe/Rome", "Europe/Madrid", "Europe/Warsaw", "Europe/Amsterdam", "Europe/Brussels", "Europe/Vienna", "Europe/Zurich", "Europe/Prague", "Europe/Budapest", "Europe/Stockholm", "Europe/Oslo", "Europe/Copenhagen", "Europe/Belgrade", "Europe/Zagreb", "Europe/Ljubljana", "Europe/Bratislava", "Europe/Luxembourg", "Europe/Malta", "Europe/Monaco", "Europe/Tirane", "Europe/Sarajevo", "Europe/Skopje", "Europe/Podgorica", "Europe/Vaduz", "Europe/Andorra", "Europe/San_Marino", "Europe/Vatican", "Europe/Busingen", "Europe/Gibraltar"],
    ),
    zone(
        EuropeAfrica,
        ["西部非洲时间", "西非時間", "West Africa Time"],
        ["拉各斯", "拉哥斯", "Lagos"],
        "Africa/Lagos",
        &["Africa/Kinshasa", "Africa/Luanda", "Africa/Douala", "Africa/Libreville", "Africa/Niamey", "Africa/Ndjamena", "Africa/Brazzaville", "Africa/Bangui", "Africa/Malabo", "Africa/Porto-Novo"],
    ),
    zone(
        EuropeAfrica,
        ["东欧时间", "東歐時間", "Eastern European Time"],
        ["基辅、雅典", "基輔、雅典", "Kyiv, Athens"],
        "Europe/Kyiv",
        &["Europe/Kiev", "Europe/Athens", "Europe/Helsinki", "Europe/Bucharest", "Europe/Sofia", "Europe/Riga", "Europe/Vilnius", "Europe/Tallinn", "Europe/Mariehamn", "Asia/Nicosia", "Europe/Nicosia", "Europe/Uzhgorod", "Europe/Zaporozhye"],
    ),
    zone(EuropeAfrica, ["埃及时间", "埃及時間", "Egypt Time"], ["开罗", "開羅", "Cairo"], "Africa/Cairo", &[]),
    zone(
        EuropeAfrica,
        ["南非标准时间", "南非標準時間", "South Africa Standard Time"],
        ["约翰内斯堡", "約翰尼斯堡", "Johannesburg"],
        "Africa/Johannesburg",
        &["Africa/Maseru", "Africa/Mbabane"],
    ),
    zone(EuropeAfrica, ["土耳其时间", "土耳其時間", "Türkiye Time"], ["伊斯坦布尔", "伊斯坦堡", "Istanbul"], "Europe/Istanbul", &["Asia/Istanbul", "Turkey"]),
    zone(
        EuropeAfrica,
        ["东部非洲时间", "東非時間", "East Africa Time"],
        ["内罗毕", "奈洛比", "Nairobi"],
        "Africa/Nairobi",
        &["Africa/Addis_Ababa", "Africa/Dar_es_Salaam", "Africa/Kampala", "Africa/Mogadishu", "Africa/Djibouti", "Africa/Asmara"],
    ),
    zone(Russia, ["加里宁格勒时间", "加里寧格勒時間", "Kaliningrad Time"], ["加里宁格勒", "加里寧格勒", "Kaliningrad"], "Europe/Kaliningrad", &[]),
    zone(
        Russia,
        ["莫斯科时间", "莫斯科時間", "Moscow Time"],
        ["莫斯科、圣彼得堡", "莫斯科、聖彼得堡", "Moscow, Saint Petersburg"],
        "Europe/Moscow",
        &["Europe/Kirov", "Europe/Volgograd", "W-SU"],
    ),
    zone(Russia, ["萨马拉时间", "薩馬拉時間", "Samara Time"], ["萨马拉", "薩馬拉", "Samara"], "Europe/Samara", &["Europe/Astrakhan", "Europe/Saratov", "Europe/Ulyanovsk"]),
    zone(Russia, ["叶卡捷琳堡时间", "葉卡捷琳堡時間", "Yekaterinburg Time"], ["叶卡捷琳堡", "葉卡捷琳堡", "Yekaterinburg"], "Asia/Yekaterinburg", &[]),
    zone(Russia, ["鄂木斯克时间", "鄂木斯克時間", "Omsk Time"], ["鄂木斯克", "鄂木斯克", "Omsk"], "Asia/Omsk", &[]),
    zone(
        Russia,
        ["克拉斯诺亚尔斯克时间", "克拉斯諾亞爾斯克時間", "Krasnoyarsk Time"],
        ["克拉斯诺亚尔斯克、新西伯利亚", "克拉斯諾亞爾斯克、新西伯利亞", "Krasnoyarsk, Novosibirsk"],
        "Asia/Krasnoyarsk",
        &["Asia/Novosibirsk", "Asia/Novokuznetsk", "Asia/Barnaul", "Asia/Tomsk"],
    ),
    zone(Russia, ["伊尔库茨克时间", "伊爾庫次克時間", "Irkutsk Time"], ["伊尔库茨克", "伊爾庫次克", "Irkutsk"], "Asia/Irkutsk", &[]),
    zone(Russia, ["雅库茨克时间", "雅庫次克時間", "Yakutsk Time"], ["雅库茨克", "雅庫次克", "Yakutsk"], "Asia/Yakutsk", &["Asia/Chita", "Asia/Khandyga"]),
    zone(Russia, ["海参崴时间", "海參崴時間", "Vladivostok Time"], ["海参崴", "海參崴", "Vladivostok"], "Asia/Vladivostok", &["Asia/Ust-Nera"]),
    zone(Russia, ["马加丹时间", "馬加丹時間", "Magadan Time"], ["马加丹", "馬加丹", "Magadan"], "Asia/Magadan", &["Asia/Sakhalin", "Asia/Srednekolymsk"]),
    zone(Russia, ["堪察加时间", "堪察加時間", "Kamchatka Time"], ["堪察加彼得罗巴甫洛夫斯克", "堪察加彼得巴甫洛夫斯克", "Petropavlovsk-Kamchatsky"], "Asia/Kamchatka", &["Asia/Anadyr"]),
    zone(Asia, ["以色列时间", "以色列時間", "Israel Time"], ["耶路撒冷", "耶路撒冷", "Jerusalem"], "Asia/Jerusalem", &["Asia/Tel_Aviv", "Israel"]),
    zone(
        Asia,
        ["阿拉伯标准时间", "阿拉伯標準時間", "Arabia Standard Time"],
        ["利雅得", "利雅德", "Riyadh"],
        "Asia/Riyadh",
        &["Asia/Kuwait", "Asia/Qatar", "Asia/Bahrain", "Asia/Baghdad", "Asia/Aden"],
    ),
    zone(Asia, ["伊朗标准时间", "伊朗標準時間", "Iran Standard Time"], ["德黑兰", "德黑蘭", "Tehran"], "Asia/Tehran", &["Iran"]),
    zone(Asia, ["海湾标准时间", "波斯灣標準時間", "Gulf Standard Time"], ["迪拜", "杜拜", "Dubai"], "Asia/Dubai", &["Asia/Muscat"]),
    zone(Asia, ["巴基斯坦标准时间", "巴基斯坦標準時間", "Pakistan Standard Time"], ["卡拉奇", "喀拉蚩", "Karachi"], "Asia/Karachi", &[]),
    zone(Asia, ["乌兹别克斯坦时间", "烏茲別克時間", "Uzbekistan Time"], ["塔什干", "塔什干", "Tashkent"], "Asia/Tashkent", &["Asia/Samarkand"]),
    zone(
        Asia,
        ["哈萨克斯坦时间", "哈薩克時間", "Kazakhstan Time"],
        ["阿拉木图、阿斯塔纳", "阿拉木圖、阿斯塔納", "Almaty, Astana"],
        "Asia/Almaty",
        &["Asia/Qyzylorda", "Asia/Aqtobe", "Asia/Aqtau", "Asia/Atyrau", "Asia/Oral", "Asia/Qostanay"],
    ),
    zone(Asia, ["印度标准时间", "印度標準時間", "India Standard Time"], ["新德里、孟买", "新德里、孟買", "New Delhi, Mumbai"], "Asia/Kolkata", &["Asia/Calcutta"]),
    zone(Asia, ["尼泊尔时间", "尼泊爾時間", "Nepal Time"], ["加德满都", "加德滿都", "Kathmandu"], "Asia/Kathmandu", &["Asia/Katmandu"]),
    zone(Asia, ["孟加拉时间", "孟加拉時間", "Bangladesh Time"], ["达卡", "達卡", "Dhaka"], "Asia/Dhaka", &["Asia/Dacca"]),
    zone(Asia, ["缅甸时间", "緬甸時間", "Myanmar Time"], ["仰光", "仰光", "Yangon"], "Asia/Yangon", &["Asia/Rangoon"]),
    zone(
        Asia,
        ["印度支那时间", "中南半島時間", "Indochina Time"],
        ["曼谷、河内", "曼谷、河內", "Bangkok, Hanoi"],
        "Asia/Bangkok",
        &["Asia/Ho_Chi_Minh", "Asia/Saigon", "Asia/Phnom_Penh", "Asia/Vientiane"],
    ),
    zone(Asia, ["印度尼西亚西部时间", "印尼西部時間", "Western Indonesia Time"], ["雅加达", "雅加達", "Jakarta"], "Asia/Jakarta", &["Asia/Pontianak"]),
    zone(
        Asia,
        ["中国标准时间", "中國標準時間", "China Standard Time"],
        ["北京、上海", "北京、上海", "Beijing, Shanghai"],
        "Asia/Shanghai",
        &["Asia/Chongqing", "Asia/Chungking", "Asia/Harbin", "PRC"],
    ),
    zone(Asia, ["香港时间", "香港時間", "Hong Kong Time"], ["香港", "香港", "Hong Kong"], "Asia/Hong_Kong", &["Hongkong"]),
    zone(Asia, ["台北时间", "台北時間", "Taipei Time"], ["台北", "台北", "Taipei"], "Asia/Taipei", &["ROC"]),
    zone(
        Asia,
        ["新加坡时间", "新加坡時間", "Singapore Time"],
        ["新加坡、吉隆坡", "新加坡、吉隆坡", "Singapore, Kuala Lumpur"],
        "Asia/Singapore",
        &["Asia/Kuala_Lumpur", "Asia/Kuching", "Singapore"],
    ),
    zone(Asia, ["菲律宾时间", "菲律賓時間", "Philippine Time"], ["马尼拉", "馬尼拉", "Manila"], "Asia/Manila", &[]),
    zone(Asia, ["印度尼西亚中部时间", "印尼中部時間", "Central Indonesia Time"], ["望加锡、登巴萨", "望加錫、登巴薩", "Makassar, Denpasar"], "Asia/Makassar", &["Asia/Ujung_Pandang"]),
    zone(Asia, ["日本标准时间", "日本標準時間", "Japan Standard Time"], ["东京", "東京", "Tokyo"], "Asia/Tokyo", &["Japan"]),
    zone(Asia, ["韩国标准时间", "韓國標準時間", "Korea Standard Time"], ["首尔", "首爾", "Seoul"], "Asia/Seoul", &["ROK"]),
    zone(Asia, ["印度尼西亚东部时间", "印尼東部時間", "Eastern Indonesia Time"], ["查亚普拉", "查亞普拉", "Jayapura"], "Asia/Jayapura", &[]),
    zone(Oceania, ["澳大利亚西部时间", "澳洲西部時間", "Australian Western Time"], ["珀斯", "伯斯", "Perth"], "Australia/Perth", &["Australia/West"]),
    zone(Oceania, ["澳大利亚中部标准时间", "澳洲中部標準時間", "Australian Central Standard Time"], ["达尔文", "達爾文", "Darwin"], "Australia/Darwin", &["Australia/North"]),
    zone(
        Oceania,
        ["澳大利亚中部时间", "澳洲中部時間", "Australian Central Time"],
        ["阿德莱德", "阿得雷德", "Adelaide"],
        "Australia/Adelaide",
        &["Australia/South", "Australia/Broken_Hill"],
    ),
    zone(
        Oceania,
        ["澳大利亚东部标准时间", "澳洲東部標準時間", "Australian Eastern Standard Time"],
        ["布里斯班", "布里斯本", "Brisbane"],
        "Australia/Brisbane",
        &["Australia/Queensland", "Australia/Lindeman"],
    ),
    zone(
        Oceania,
        ["澳大利亚东部时间", "澳洲東部時間", "Australian Eastern Time"],
        ["悉尼、墨尔本", "雪梨、墨爾本", "Sydney, Melbourne"],
        "Australia/Sydney",
        &["Australia/Melbourne", "Australia/Canberra", "Australia/Hobart", "Australia/ACT", "Australia/NSW", "Australia/Victoria", "Australia/Tasmania"],
    ),
    zone(Oceania, ["新西兰时间", "紐西蘭時間", "New Zealand Time"], ["奥克兰", "奧克蘭", "Auckland"], "Pacific/Auckland", &["NZ"]),
    Zone {
        name: ["协调世界时", "世界標準時間", "Coordinated Universal Time"],
        cities: ["UTC", "UTC", "UTC"],
        tz: UTC,
        also: &["Etc/UTC", "Etc/UCT", "UCT", "Etc/Universal", "Universal", "Etc/Zulu", "Zulu", "Etc/GMT", "GMT", "Etc/Greenwich", "Greenwich"],
        region: None,
    },
]
};


/// Where the zone the tz database names `tz` is in [`TIME_ZONES`], by its
/// own name or one of those that keep its time under its name.
pub fn listed(tz: &str) -> Option<usize> {
    TIME_ZONES.iter().position(|zone| zone.tz == tz || zone.also.contains(&tz))
}

/// The time zone the installed system is set to.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemZone {
    /// Its name in the tz database.
    pub name: String,
    /// The rules the recovery carries for it.
    pub rules: TimeZone,
}

impl SystemZone {
    /// The zone the tz database names `name`, with the rules the recovery
    /// carries for it.
    pub fn named(name: &str) -> Result<SystemZone> {
        let rules = TimeZone::get(name).with_context(|| format!("the recovery carries no rules for the time zone {name:?}"))?;
        Ok(SystemZone { name: name.to_string(), rules })
    }
}

impl State {
    /// The installed system's time zone, while the clock keeps it.
    pub fn followed(&self) -> Option<&SystemZone> {
        self.system_zone.as_ref().filter(|_| self.follow_system_zone)
    }

    /// The rules of the zone the clock and the times on the pages keep: the
    /// installed system's while it is followed and known, and the one chosen
    /// otherwise.
    pub fn zone(&self) -> &TimeZone {
        match self.followed() {
            Some(system) => &system.rules,
            None => fmt::rules(self.time_zone),
        }
    }

    /// Where the zone the clock keeps is in [`TIME_ZONES`]; none for a zone
    /// of the installed system's that is not listed.
    pub fn zone_listed(&self) -> Option<usize> {
        match self.followed() {
            Some(system) => listed(&system.name),
            None => Some(self.time_zone),
        }
    }

    /// What the zone the clock keeps is called: its name where it is listed,
    /// and otherwise the name the tz database gives it.
    pub fn zone_name(&self) -> &str {
        match (self.zone_listed(), self.followed()) {
            (Some(at), _) => self.lang.pick(TIME_ZONES[at].name),
            (None, Some(system)) => &system.name,
            (None, None) => unreachable!("a zone chosen is one listed"),
        }
    }
}

impl App {
    /// The installed system is set to the time zone the tz database names
    /// `name`: the clock keeps it while the system's zone is followed.
    pub fn set_system_zone(&mut self, name: &str) -> Result<()> {
        let zone = SystemZone::named(name)?;
        if self.state.system_zone.as_ref() != Some(&zone) {
            self.state.system_zone = Some(zone);
            self.refresh();
        }
        Ok(())
    }

    /// The time is `unix`, in Unix seconds. The time zone pages tell how far
    /// each zone is from UTC now, to the minute, and are drawn again when a
    /// zone's clocks go forward or back.
    pub fn set_now(&mut self, unix: i64) {
        let minute = unix - unix.rem_euclid(60);
        if minute == self.state.now {
            return;
        }
        let before = self.offsets();
        self.state.now = minute;
        if matches!(self.screen(), Screen::TimeZone | Screen::TimeZones(_)) && self.offsets() != before {
            self.refresh();
        }
    }

    /// How far from UTC each zone the time zone pages tell of is now.
    fn offsets(&self) -> Vec<i32> {
        let now = self.state.now;
        let listed = (0..TIME_ZONES.len()).map(|at| fmt::utc_offset_seconds(now, fmt::rules(at)));
        listed.chain(self.state.system_zone.iter().map(|system| fmt::utc_offset_seconds(now, &system.rules))).collect()
    }
}


impl App {
    pub(super) fn set_slider(&mut self, t: &Theme, which: Slider, track: Rect, x: f32) {
        self.set_level(t, which, track, (x - track.x) / track.w);
    }

    /// Where a slider stands, as a share of its span.
    pub(super) fn level(&self, which: Slider) -> f32 {
        match which {
            Slider::Brightness => self.state.brightness,
            Slider::Vibration => self.state.vibration,
            Slider::ThermalLimit => {
                let span = f32::from(THERMAL_LIMIT_MOST - THERMAL_LIMIT_LEAST);
                (f32::from(self.state.thermal_limit - THERMAL_LIMIT_LEAST) / span).clamp(0.0, 1.0)
            }
            which => self.palette_level(which).unwrap_or(0.0),
        }
    }

    /// 主题颜色页上的滑杆：主色 / 页面底色 / 磁贴底色各三个分量，加壁纸那五根。
    fn palette_level(&self, which: Slider) -> Option<f32> {
        let s = &self.state;
        let (ar, ag, ab) = accent_of(s.theme, s.accent);
        let (br, bg, bb) = background_of(s.theme, s.background);
        let (tr, tg, tb) = tile_of(s.theme, s.tile_tint);
        Some(match which {
            Slider::Foreground => s.foreground,
            Slider::WallpaperDim => s.wallpaper_dim,
            Slider::WallpaperZoom => ((s.wallpaper_zoom - 1.0) / 3.0).clamp(0.0, 1.0),
            Slider::WallpaperPanX => s.wallpaper_pan_x,
            Slider::WallpaperPanY => s.wallpaper_pan_y,
            Slider::AccentRed => f32::from(ar) / 255.0,
            Slider::AccentGreen => f32::from(ag) / 255.0,
            Slider::AccentBlue => f32::from(ab) / 255.0,
            Slider::BackRed => f32::from(br) / 255.0,
            Slider::BackGreen => f32::from(bg) / 255.0,
            Slider::BackBlue => f32::from(bb) / 255.0,
            Slider::TileRed => f32::from(tr) / 255.0,
            Slider::TileGreen => f32::from(tg) / 255.0,
            Slider::TileBlue => f32::from(tb) / 255.0,
            _ => return None,
        })
    }

    /// 主题颜色页上的一根滑杆被拖到 `level`（0..1）。
    fn set_palette_level(&mut self, t: &Theme, which: Slider, track: Rect, level: f32) {
        let lv = level.clamp(0.0, 1.0);
        let byte = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
        let mut reshape = false;
        match which {
            Slider::Foreground => self.state.foreground = lv,
            Slider::WallpaperDim => self.state.wallpaper_dim = lv,
            Slider::WallpaperZoom => {
                self.state.wallpaper_zoom = 1.0 + lv * 3.0;
                reshape = true;
            }
            Slider::WallpaperPanX => {
                self.state.wallpaper_pan_x = lv;
                reshape = true;
            }
            Slider::WallpaperPanY => {
                self.state.wallpaper_pan_y = lv;
                reshape = true;
            }
            Slider::AccentRed | Slider::AccentGreen | Slider::AccentBlue => {
                let (mut r, mut g, mut b) = accent_of(self.state.theme, self.state.accent);
                match which {
                    Slider::AccentRed => r = byte(lv),
                    Slider::AccentGreen => g = byte(lv),
                    _ => b = byte(lv),
                }
                self.state.accent = Some((r, g, b));
            }
            Slider::BackRed | Slider::BackGreen | Slider::BackBlue => {
                let (mut r, mut g, mut b) = background_of(self.state.theme, self.state.background);
                match which {
                    Slider::BackRed => r = byte(lv),
                    Slider::BackGreen => g = byte(lv),
                    _ => b = byte(lv),
                }
                self.state.background = Some((r, g, b));
            }
            Slider::TileRed | Slider::TileGreen | Slider::TileBlue => {
                let (mut r, mut g, mut b) = tile_of(self.state.theme, self.state.tile_tint);
                match which {
                    Slider::TileRed => r = byte(lv),
                    Slider::TileGreen => g = byte(lv),
                    _ => b = byte(lv),
                }
                self.state.tile_tint = Some((r, g, b));
            }
            _ => {}
        }
        // 缩放与位置要按原图重新采样；拖动中间不重算，等松手（`settle_slider`）。
        if reshape {
            self.state.paper_dirty = true;
        }
        if let Some(fr) = self.frame.as_mut() {
            fr.canvas.fill_rect(track.x, track.y, track.w, track.h, t.pal.card);
            widgets::slider(&mut fr.canvas, t, track.x, track.y, track.w, lv);
        }
        self.layer = None;
        self.dirty = true;
    }

    /// 滑杆松手：壁纸按新的缩放与位置重采样一次（拖动中间不重算，
    /// 三百多万像素一采样就是一两百毫秒）。
    pub(super) fn settle_slider(&mut self, which: Slider) {
        if !matches!(which, Slider::WallpaperZoom | Slider::WallpaperPanX | Slider::WallpaperPanY) {
            return;
        }
        if self.state.paper_dirty {
            self.state.paper_dirty = false;
            self.reshape_wallpaper();
        }
    }

    /// Set a slider to `level` of its span, the brightness no lower than
    /// its floor, and draw the slider at `track` on the page again.
    pub(super) fn set_level(&mut self, t: &Theme, which: Slider, track: Rect, level: f32) {
        if !matches!(which, Slider::Brightness | Slider::Vibration | Slider::ThermalLimit) {
            self.set_palette_level(t, which, track, level);
            return;
        }
        if which == Slider::ThermalLimit {
            // The limit is a whole degree, so the slider lands on one rather
            // than between two: the span is short, and half a degree either way
            // would be a limit the page could not name.
            let span = f32::from(THERMAL_LIMIT_MOST - THERMAL_LIMIT_LEAST);
            let step = (level.clamp(0.0, 1.0) * span).round() as u8;
            self.state.thermal_limit = (THERMAL_LIMIT_LEAST + step).min(THERMAL_LIMIT_MOST);
            let value = self.level(which);
            if let Some(fr) = self.frame.as_mut() {
                fr.canvas.fill_rect(track.x, track.y, track.w, track.h, t.pal.card);
                widgets::slider(&mut fr.canvas, t, track.x, track.y, track.w, value);
            }
            self.layer = None;
            self.dirty = true;
            return;
        }
        let value = match which {
            Slider::Brightness => &mut self.state.brightness,
            Slider::Vibration => &mut self.state.vibration,
            Slider::ThermalLimit => unreachable!("the battery's limit is set above"),
            _ => unreachable!("the colours and the wallpaper are set above"),
        };
        *value = level.clamp(if which == Slider::Brightness { MIN_BRIGHTNESS } else { 0.0 }, 1.0);
        let value = *value;
        if let Some(fr) = self.frame.as_mut() {
            fr.canvas.fill_rect(track.x, track.y, track.w, track.h, t.pal.card);
            widgets::slider(&mut fr.canvas, t, track.x, track.y, track.w, value);
        }
        self.layer = None;
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::testing::*;

    #[test]
    fn the_file_transfer_switch_has_the_runtime_follow_it() {
        let (f, i) = (fonts(), info());
        // 文件传输搬进了自己的页，开关在那页上（2026-11-28）。
        let mut app = App::new(Screen::FileTransfer, W, H, PPD);
        assert!(app.state.file_transfer, "on until turned off");
        let (_, y) = spot(&mut app, &f, &i, Action::Toggle(Switch::FileTransfer));
        if y > H as f32 * 0.9 {
            scroll_to(&mut app, &f, y - H as f32 * 0.8, 0);
        }
        assert_eq!(tap_at(&mut app, &f, &i, Action::Toggle(Switch::FileTransfer), 2000), [Effect::FileTransfer(false)]);
        assert!(!app.state.file_transfer);
        assert_eq!(tap_at(&mut app, &f, &i, Action::Toggle(Switch::FileTransfer), 3000), [Effect::FileTransfer(true)]);
        app.file_transfer_failed(false, "无法开启文件传输", 4000);
        assert!(!app.state.file_transfer, "the switch shows how it stands");
    }

    #[test]
    fn theme_previews_switch_the_theme() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Settings, W, H, PPD);
        tap(&mut app, &f, &i, Action::Theme { dark: true });
        assert!(app.state.dark);
        assert!(app.theme().dark);
        tap(&mut app, &f, &i, Action::Theme { dark: false });
        assert!(!app.state.dark);
    }

    #[test]
    fn dragging_the_track_sets_brightness() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Settings, W, H, PPD);
        let track = app.frame(&f, &i).hits.iter().find(|h| h.action == Action::Slider(Slider::Brightness)).unwrap().rect;
        let y = track.centre().1 - app.top().1;
        assert!(app.touch(Touch::Down(track.x + track.w * 0.25, y), &f, 0).is_empty());
        assert_eq!(app.panel(0), Some(0.25));
        assert!(app.touch(Touch::Move(track.x + track.w * 0.75, y + 40.0), &f, 0).is_empty());
        assert_eq!(app.panel(0), Some(0.75));
        assert!(app.touch(Touch::Up, &f, 0).is_empty(), "the panel shows the brightness");
        assert_eq!(app.state.brightness, 0.75);
    }

    #[test]
    fn the_strength_slider_taps_the_motor_as_strongly_as_it_is_let_go() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Settings, W, H, PPD);
        let track = app.frame(&f, &i).hits.iter().find(|h| h.action == Action::Slider(Slider::Vibration)).unwrap().rect;
        let y = track.centre().1 - app.top().1;
        assert!(app.touch(Touch::Down(track.x + track.w * 0.25, y), &f, 0).is_empty());
        assert_eq!(app.touch(Touch::Move(track.x + track.w * 0.8, y + 40.0), &f, 0), vec![Effect::Haptic], "the step crossed is a tap");
        assert!(app.touch(Touch::Move(track.x + track.w * 0.8, y + 40.0), &f, 0).is_empty(), "the step it stands on again is silent");
        assert_eq!(app.touch(Touch::Up, &f, 0), vec![Effect::Haptic], "let go, the motor taps");
        assert!((app.state.vibration - 0.8).abs() < 1e-5);
        assert!(app.touch(Touch::Down(track.x - 50.0, y), &f, 0).is_empty());
        assert!(app.touch(Touch::Up, &f, 0).is_empty(), "touched beside the track, the slider stays");
        app.state.vibrate = false;
        app.touch(Touch::Down(track.x, y), &f, 0);
        assert!(app.touch(Touch::Up, &f, 0).is_empty(), "with touch vibration off, the motor stays still");
        assert_eq!(app.state.vibration, 0.0, "the strength still moves, down to the slider's left end");
    }

    #[test]
    fn the_language_chosen_is_spoken_on_every_page() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Settings, &f, &i);
        tap(&mut app, &f, &i, Action::Open(Screen::Language));
        assert_eq!(app.frame(&f, &i).title, "语言");
        tap(&mut app, &f, &i, Action::Language(Lang::En));
        assert_eq!((app.state.lang, app.screen()), (Lang::En, Screen::Language), "the page stays");
        assert_eq!(app.frame(&f, &i).title, "Language");
        tap(&mut app, &f, &i, Action::Language(Lang::Hant));
        assert_eq!(app.frame(&f, &i).title, "語言");
        tap_back(&mut app, &f, &i, 0);
        app.tick(1000);
        assert_eq!((app.screen(), app.frame(&f, &i).title.to_string()), (Screen::Settings, "選項設定".to_string()), "and so does the page gone back to");
        app.set_mounts(Ok(vec![MountRow { base: "odm".into(), point: "/odm".into(), mounted: true, holders: vec!["qseecomd".into()] }]));
        tap(&mut app, &f, &i, Action::Open(Screen::Language));
        tap(&mut app, &f, &i, Action::Language(Lang::En));
        tap_back(&mut app, &f, &i, 0);
        app.tick(2000);
        tap_back(&mut app, &f, &i, 2000);
        app.tick(3000);
        // The mount page is behind its own home entry now.
        tap(&mut app, &f, &i, Action::Open(Screen::Settings));
        tap(&mut app, &f, &i, Action::Open(Screen::Mount));
        tap(&mut app, &f, &i, Action::Unavailable("A partition in use can't be unmounted"));
        assert_eq!(app.toast.as_ref().map(|t| t.0.as_str()), Some("A partition in use can't be unmounted"), "as do its messages");
    }

    #[test]
    fn a_language_chosen_is_kept_and_spoken_over_the_one_the_system_names() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Settings, &f, &i);
        app.take_settings(None, Some(Lang::Hant));
        assert_eq!((app.state.lang, app.settings().language), (Lang::Hant, None), "the system's, until one is chosen");
        tap(&mut app, &f, &i, Action::Open(Screen::Language));
        tap(&mut app, &f, &i, Action::Language(Lang::En));
        let kept = app.settings();
        assert_eq!(kept.language.as_deref(), Some("en"));

        let mut next = App::new(Screen::Home, W, H, PPD);
        next.take_settings(Some(&kept), Some(Lang::Hans));
        assert_eq!(next.state.lang, Lang::En, "the next start speaks the language chosen");
        assert_eq!(next.settings(), kept, "and keeps it chosen");
    }

    #[test]
    fn with_no_language_chosen_a_start_speaks_the_system_s_or_else_the_one_last_spoken() {
        let spoke_english = Settings { spoken: Some("en".into()), ..Settings::default() };
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.take_settings(Some(&spoke_english), Some(Lang::Hant));
        assert_eq!(app.state.lang, Lang::Hant, "the one the system names");
        let mut fastboot = App::new(Screen::Fastboot, W, H, PPD);
        fastboot.take_settings(Some(&spoke_english), None);
        assert_eq!(fastboot.state.lang, Lang::En, "a start that names none, as fastboot's");
        let mut first = App::new(Screen::Fastboot, W, H, PPD);
        first.take_settings(None, None);
        assert_eq!(first.state.lang, State::default().lang, "and with nothing kept, the pages' own");
    }

    #[test]
    fn the_other_pages_choices_are_kept_too() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Settings, &f, &i);
        tap(&mut app, &f, &i, Action::Toggle(Switch::Vibrate));
        app.state.check_signature = true;
        app.state.backup_to = Place::UsbDrive;
        app.state.logs_to = 1;
        let kept = app.settings();
        let mut next = App::new(Screen::Home, W, H, PPD);
        next.take_settings(Some(&kept), None);
        assert!(!next.state.vibrate && next.state.check_signature);
        assert_eq!((next.state.backup_to, next.state.logs_to), (Place::UsbDrive, 1));
    }

    #[test]
    fn logs_are_exported_as_ticked_where_they_can_go() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Logs, W, H, PPD);
        let both = Effect::ExportLogs { recovery: true, kernel: true, storage: false };
        assert_eq!(tap(&mut app, &f, &i, Action::ExportLogs), vec![both], "both, to the temporary folder, at first");
        tap(&mut app, &f, &i, Action::LogPart(0));
        tap(&mut app, &f, &i, Action::LogsTo(1));
        assert!(tap(&mut app, &f, &i, Action::ExportLogs).is_empty(), "internal storage waits for the data partition");
        app.tick(1000);
        assert_eq!(app.deadline(1000), Some(TOAST_MS), "and says so");
        app.data_waiting(None);
        assert_eq!(tap(&mut app, &f, &i, Action::ExportLogs), vec![Effect::ExportLogs { recovery: false, kernel: true, storage: true }]);
        tap(&mut app, &f, &i, Action::LogPart(1));
        assert!(tap(&mut app, &f, &i, Action::ExportLogs).is_empty(), "nothing ticked, nothing saved");
    }

    #[test]
    fn a_time_zone_chosen_is_the_one_kept() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Settings, W, H, PPD);
        tap(&mut app, &f, &i, Action::Open(Screen::TimeZone));
        assert_eq!(app.state.time_zone, 0, "China until another is chosen");
        let london = listed("Europe/London").expect("London is listed");
        tap(&mut app, &f, &i, Action::TimeZone(london));
        assert_eq!(app.state.time_zone, london);
        tap(&mut app, &f, &i, Action::TimeZone(0));
        assert_eq!(app.state.time_zone, 0, "and China again");
    }

    #[test]
    fn choosing_how_long_the_screen_stays_on() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Settings, &f, &i);
        tap(&mut app, &f, &i, Action::Open(Screen::ScreenOff));
        assert_eq!(app.screen(), Screen::ScreenOff);
        let frame = app.frame(&f, &i);
        assert_eq!(frame.title, "息屏时间");
        assert!(frame.back);
        // The choices sit in order, top to bottom.
        let actions: Vec<Action> = frame.hits.iter().map(|h| h.action).collect();
        assert_eq!(actions, (0..SCREEN_OFF.len()).map(Action::ScreenOff).collect::<Vec<_>>());
        tap(&mut app, &f, &i, Action::ScreenOff(0));
        assert_eq!(app.state.screen_off, 0);
        assert_eq!(app.screen(), Screen::ScreenOff, "the page stays");
        // Fifteen seconds, dim for the last three, once the row tapped has
        // faded back.
        app.tick(1000);
        assert_eq!(app.deadline(1000), Some(12_000));
        app.tick(15_000);
        assert!(!app.display_on());
    }
}
