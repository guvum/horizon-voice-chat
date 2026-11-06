//! The game's own look: vanilla buttons, sliders and text fields drawn from
//! the GUI atlas by name (nothing of vanilla's is shipped), the mod's own
//! icons, and a plain fallback for a Horizon whose atlas lacks a sprite.
//!
//! Sprite names are the atlas keys Horizon gives the vanilla GUI sprites
//! (`gui/sprites/widget/button`, from `assets/minecraft/textures/gui/sprites/
//! widget/button.png`) and a content mod's GUI textures (`voicechat:gui/icons/
//! microphone`, from `assets/voicechat/textures/gui/icons/microphone.png`).
//! The sizes and borders are vanilla's `.png.mcmeta` nine-slice values, the
//! same Horizon falls back to (`horizon-client-render/src/ui.rs`, `scaling`).
use crate::icons;
use crate::ui::Draw;

/// A nine-slice sprite: its GUI size and its borders (left, top, right, bottom).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NineSlice {
    pub name: &'static str,
    pub size: [f32; 2],
    pub border: [f32; 4],
}

impl NineSlice {
    const fn even(name: &'static str, size: [f32; 2], border: f32) -> Self {
        Self { name, size, border: [border; 4] }
    }
}

pub const BUTTON: NineSlice = NineSlice::even("gui/sprites/widget/button", [200.0, 20.0], 3.0);
pub const BUTTON_HIGHLIGHTED: NineSlice = NineSlice::even("gui/sprites/widget/button_highlighted", [200.0, 20.0], 3.0);
pub const BUTTON_DISABLED: NineSlice = NineSlice::even("gui/sprites/widget/button_disabled", [200.0, 20.0], 3.0);
pub const SLIDER: NineSlice = NineSlice::even("gui/sprites/widget/slider", [200.0, 20.0], 1.0);
pub const SLIDER_HIGHLIGHTED: NineSlice = NineSlice::even("gui/sprites/widget/slider_highlighted", [200.0, 20.0], 1.0);
pub const SLIDER_HANDLE: NineSlice =
    NineSlice { name: "gui/sprites/widget/slider_handle", size: [8.0, 20.0], border: [2.0, 2.0, 2.0, 3.0] };
pub const SLIDER_HANDLE_HIGHLIGHTED: NineSlice =
    NineSlice { name: "gui/sprites/widget/slider_handle_highlighted", size: [8.0, 20.0], border: [2.0, 2.0, 2.0, 3.0] };
pub const TEXT_FIELD: NineSlice = NineSlice::even("gui/sprites/widget/text_field", [200.0, 20.0], 1.0);
pub const TEXT_FIELD_HIGHLIGHTED: NineSlice = NineSlice::even("gui/sprites/widget/text_field_highlighted", [200.0, 20.0], 1.0);

/// Every vanilla sprite the mod draws: checked once, so a renamed one is in the log.
pub const REQUIRED: [&str; 9] = [
    BUTTON.name,
    BUTTON_HIGHLIGHTED.name,
    BUTTON_DISABLED.name,
    SLIDER.name,
    SLIDER_HIGHLIGHTED.name,
    SLIDER_HANDLE.name,
    SLIDER_HANDLE_HIGHLIGHTED.name,
    TEXT_FIELD.name,
    TEXT_FIELD_HIGHLIGHTED.name,
];

/// Vanilla's label colours: white, grey when disabled.
pub const LABEL: u32 = 0xFF_FFFF;
pub const LABEL_DISABLED: u32 = 0xA0_A0A0;

fn sprite(name: &str, src: [f32; 4], dst: [f32; 4]) -> Draw {
    Draw::Sprite { name: name.to_owned(), src, dst }
}

/// The source span `[s, len]` repeated across the destination span `[d, len]`
/// (vanilla tiles a nine-slice's edges and centre): `(src, dst)` pairs.
fn tile(src: [f32; 2], dst: [f32; 2]) -> Vec<([f32; 2], [f32; 2])> {
    if src[1] <= 0.0 || dst[1] <= 0.0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut at = 0.0;
    while at < dst[1] {
        let len = (dst[1] - at).min(src[1]);
        out.push(([src[0], len], [dst[0] + at, len]));
        at += len;
    }
    out
}

/// `s` drawn into `dst`: corners as they are, edges and centre tiled, as
/// vanilla's `blitNineSlicedSprite`. At least nine parts; more when `dst` is
/// larger than the sprite.
pub fn nine_slice(s: &NineSlice, dst: [f32; 4]) -> Vec<Draw> {
    let [x, y, w, h] = dst;
    let [sw, sh] = s.size;
    let (half_w, half_h) = ((w / 2.0).floor(), (h / 2.0).floor());
    let [l, t, r, b] = s.border;
    let (l, r, t, b) = (l.min(half_w), r.min(half_w), t.min(half_h), b.min(half_h));
    // Columns and rows: (source start, source length, destination start, destination length).
    let cols = [(0.0, l, x, l), (l, sw - l - r, x + l, w - l - r), (sw - r, r, x + w - r, r)];
    let rows = [(0.0, t, y, t), (t, sh - t - b, y + t, h - t - b), (sh - b, b, y + h - b, b)];
    let mut out = Vec::new();
    for &(sy, sl_y, dy, dl_y) in &rows {
        for &(sx, sl_x, dx, dl_x) in &cols {
            // Corners keep their size; edges and the centre tile.
            let xs = if sl_x == dl_x { vec![([sx, sl_x], [dx, dl_x])] } else { tile([sx, sl_x], [dx, dl_x]) };
            let ys = if sl_y == dl_y { vec![([sy, sl_y], [dy, dl_y])] } else { tile([sy, sl_y], [dy, dl_y]) };
            for (src_y, dst_y) in &ys {
                for (src_x, dst_x) in &xs {
                    if dst_x[1] > 0.0 && dst_y[1] > 0.0 {
                        out.push(sprite(s.name, [src_x[0], src_y[0], src_x[1], src_y[1]], [dst_x[0], dst_y[0], dst_x[1], dst_y[1]]));
                    }
                }
            }
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Disabled,
}

/// `label` cut to `max` pixels, with an ellipsis when it was longer.
pub fn fit(label: &str, max: f32, text_width: &dyn Fn(&str) -> f32) -> String {
    if text_width(label) <= max {
        return label.to_owned();
    }
    let mut cut: Vec<char> = label.chars().collect();
    while !cut.is_empty() && text_width(&format!("{}…", cut.iter().collect::<String>())) > max {
        cut.pop();
    }
    format!("{}…", cut.iter().collect::<String>().trim_end())
}

/// A vanilla button with `label` centred; `text_width` is the label's width.
pub fn button(rect: [f32; 4], state: ButtonState, label: &str, text_width: f32) -> Vec<Draw> {
    let s = match state {
        ButtonState::Normal => &BUTTON,
        ButtonState::Hover => &BUTTON_HIGHLIGHTED,
        ButtonState::Disabled => &BUTTON_DISABLED,
    };
    let mut out = nine_slice(s, rect);
    if !label.is_empty() {
        let [x, y, w, h] = rect;
        let rgb = if state == ButtonState::Disabled { LABEL_DISABLED } else { LABEL };
        out.push(Draw::Text(label.to_owned(), [(x + (w - text_width) / 2.0).floor(), y + ((h - 8.0) / 2.0).floor()], rgb, true));
    }
    out
}

/// A button showing `icon` (16×16) in its middle instead of a label.
pub fn icon_button(rect: [f32; 4], state: ButtonState, i: Icon) -> Vec<Draw> {
    let mut out = button(rect, state, "", 0.0);
    let [x, y, w, h] = rect;
    out.push(icon(i, [(x + (w - 16.0) / 2.0).floor(), (y + (h - 16.0) / 2.0).floor(), 16.0, 16.0]));
    out
}

/// The handle's left edge for `value` (0..=1) on a slider at `rect`.
pub fn handle_x(rect: [f32; 4], value: f32) -> f32 {
    rect[0] + (value.clamp(0.0, 1.0) * (rect[2] - 8.0)).floor()
}

/// The value (0..=1) a click at `x` picks on a slider at `rect`
/// (`AbstractSliderButton.setValueFromMouse`).
pub fn slider_value(rect: [f32; 4], x: f32) -> f32 {
    ((x - (rect[0] + 4.0)) / (rect[2] - 8.0)).clamp(0.0, 1.0)
}

/// A vanilla slider: the track, the 8 px handle at `value` (0..=1) and the label.
pub fn slider(rect: [f32; 4], value: f32, hover: bool, label: &str, text_width: f32) -> Vec<Draw> {
    let [x, y, w, h] = rect;
    let mut out = nine_slice(&SLIDER, rect);
    out.extend(slider_handle(rect, value, hover));
    out.push(Draw::Text(label.to_owned(), [(x + (w - text_width) / 2.0).floor(), y + ((h - 8.0) / 2.0).floor()], LABEL, true));
    out
}

/// Only a slider's handle, for sliders that draw something between the track and it.
pub fn slider_handle(rect: [f32; 4], value: f32, hover: bool) -> Vec<Draw> {
    let handle = if hover { &SLIDER_HANDLE_HIGHLIGHTED } else { &SLIDER_HANDLE };
    nine_slice(handle, [handle_x(rect, value), rect[1], 8.0, rect[3]])
}

/// A vanilla text field's frame (the text goes on top).
pub fn text_field(rect: [f32; 4], focused: bool) -> Vec<Draw> {
    nine_slice(if focused { &TEXT_FIELD_HIGHLIGHTED } else { &TEXT_FIELD }, rect)
}

/// A tooltip as vanilla draws one: a dark box with a purple frame, `lines`
/// from its top left, kept on a `size` screen next to the pointer at `at`.
pub fn tooltip(lines: &[String], at: [f32; 2], size: [f32; 2], text_width: &dyn Fn(&str) -> f32) -> Vec<Draw> {
    if lines.is_empty() {
        return Vec::new();
    }
    let w = lines.iter().map(|l| text_width(l)).fold(0.0, f32::max);
    let h = lines.len() as f32 * 10.0 - 2.0;
    let mut x = at[0] + 12.0;
    let mut y = at[1] - 12.0;
    if x + w + 4.0 > size[0] {
        x = (at[0] - 16.0 - w).max(4.0);
    }
    y = y.clamp(4.0, (size[1] - h - 4.0).max(4.0));
    let bg = 0xF010_0010;
    let mut out = vec![
        Draw::Fill([x - 3.0, y - 4.0, w + 6.0, 1.0], bg),
        Draw::Fill([x - 3.0, y + h + 3.0, w + 6.0, 1.0], bg),
        Draw::Fill([x - 3.0, y - 3.0, w + 6.0, h + 6.0], bg),
        Draw::Fill([x - 4.0, y - 3.0, 1.0, h + 6.0], bg),
        Draw::Fill([x + w + 3.0, y - 3.0, 1.0, h + 6.0], bg),
        Draw::Fill([x - 3.0, y - 2.0, 1.0, h + 4.0], 0x5050_00FF),
        Draw::Fill([x + w + 2.0, y - 2.0, 1.0, h + 4.0], 0x5028_007F),
        Draw::Fill([x - 3.0, y - 3.0, w + 6.0, 1.0], 0x5050_00FF),
        Draw::Fill([x - 3.0, y + h + 2.0, w + 6.0, 1.0], 0x5028_007F),
    ];
    for (i, line) in lines.iter().enumerate() {
        out.push(Draw::Text(line.clone(), [x, y + i as f32 * 10.0], LABEL, true));
    }
    out
}

/// A light grey vanilla container panel (as an inventory's): the dark title
/// colour goes with it.
pub fn panel(rect: [f32; 4]) -> Vec<Draw> {
    let [x, y, w, h] = rect;
    vec![
        Draw::Fill([x + 1.0, y, w - 2.0, h], 0xFF00_0000),
        Draw::Fill([x, y + 1.0, w, h - 2.0], 0xFF00_0000),
        Draw::Fill([x + 1.0, y + 1.0, w - 2.0, h - 2.0], 0xFFFF_FFFF),
        Draw::Fill([x + 2.0, y + 2.0, w - 3.0, h - 3.0], 0xFF55_5555),
        Draw::Fill([x + 2.0, y + 2.0, w - 4.0, h - 4.0], 0xFFC6_C6C6),
    ]
}

/// A sunken dark area inside a panel (a list's background).
pub fn inset(rect: [f32; 4]) -> Vec<Draw> {
    let [x, y, w, h] = rect;
    vec![
        Draw::Fill([x, y, w, h], 0xFF37_3737),
        Draw::Fill([x + 1.0, y + 1.0, w - 1.0, h - 1.0], 0xFFFF_FFFF),
        Draw::Fill([x + 1.0, y + 1.0, w - 2.0, h - 2.0], 0xFF8B_8B8B),
    ]
}

/// The title colour on a panel.
pub const TITLE: u32 = 0x40_4040;

/// A list entry's background (Simple Voice Chat's `BG_FILL`), hovered, and
/// chosen.
pub const ENTRY: u32 = 0xFF4A_4A4A;
pub const ENTRY_HOVER: u32 = 0xFF5A_5A5A;
pub const ENTRY_CHOSEN: u32 = 0xFF28_2828;

/// Where a player's face goes: the game gives mods no skins to draw, so a
/// square in a colour of their own with their name's first letter.
pub fn head(rect: [f32; 4], name: &str, id: u128, text_width: &dyn Fn(&str) -> f32) -> Vec<Draw> {
    const COLOURS: [u32; 8] =
        [0xFF8E_5A3C, 0xFF3C_6E8E, 0xFF5A_8E3C, 0xFF8E_3C6E, 0xFF6E_5A8E, 0xFF3C_8E7A, 0xFF8E_7A3C, 0xFF7A_3C3C];
    let [x, y, w, h] = rect;
    let colour = COLOURS[((id ^ (id >> 64)) % COLOURS.len() as u128) as usize];
    let mut out = vec![Draw::Fill(rect, 0xFF20_2020), Draw::Fill([x + 1.0, y + 1.0, w - 2.0, h - 2.0], colour)];
    if let Some(c) = name.chars().find(|c| c.is_alphanumeric()) {
        let letter: String = c.to_uppercase().collect();
        let lw = text_width(&letter);
        out.push(Draw::Text(letter, [(x + (w - lw) / 2.0).round(), (y + (h - 8.0) / 2.0).round()], 0xFF_FFFF, true));
    }
    out
}

/// The frame around a talking player's face (`talk_outline`), `t` thick.
pub fn talk_outline(rect: [f32; 4], t: f32) -> Vec<Draw> {
    let [x, y, w, h] = rect;
    let c = 0xFF55_FF55;
    vec![
        Draw::Fill([x, y, w, t], c),
        Draw::Fill([x, y + h - t, w, t], c),
        Draw::Fill([x, y + t, t, h - 2.0 * t], c),
        Draw::Fill([x + w - t, y + t, t, h - 2.0 * t], c),
    ]
}

/// The mod's own icons, 16×16, drawn by this project (MIT): `make_icons.py`
/// writes them as PNGs from the same pixels as [`icons`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Mic,
    MicOff,
    Speaker,
    SpeakerOff,
    Whisper,
    Disconnected,
    Group,
    Settings,
    Lock,
    Volumes,
    Eye,
    EyeOff,
    Leave,
    GroupHud,
    GroupHudOff,
    Check,
    Search,
}

impl Icon {
    pub const ALL: [Icon; 17] = [
        Icon::Mic,
        Icon::MicOff,
        Icon::Speaker,
        Icon::SpeakerOff,
        Icon::Whisper,
        Icon::Disconnected,
        Icon::Group,
        Icon::Settings,
        Icon::Lock,
        Icon::Volumes,
        Icon::Eye,
        Icon::EyeOff,
        Icon::Leave,
        Icon::GroupHud,
        Icon::GroupHudOff,
        Icon::Check,
        Icon::Search,
    ];

    /// The PNG's file name, without `.png`.
    pub fn file(self) -> &'static str {
        match self {
            Icon::Mic => "microphone",
            Icon::MicOff => "microphone_off",
            Icon::Speaker => "speaker",
            Icon::SpeakerOff => "speaker_off",
            Icon::Whisper => "whisper",
            Icon::Disconnected => "disconnected",
            Icon::Group => "group",
            Icon::Settings => "settings",
            Icon::Lock => "lock",
            Icon::Volumes => "volumes",
            Icon::Eye => "eye",
            Icon::EyeOff => "eye_off",
            Icon::Leave => "leave",
            Icon::GroupHud => "group_hud",
            Icon::GroupHudOff => "group_hud_off",
            Icon::Check => "check",
            Icon::Search => "search",
        }
    }

    /// Its atlas name, from the content mod's `assets/voicechat/textures/gui/icons/`.
    pub fn sprite(self) -> &'static str {
        match self {
            Icon::Mic => "voicechat:gui/icons/microphone",
            Icon::MicOff => "voicechat:gui/icons/microphone_off",
            Icon::Speaker => "voicechat:gui/icons/speaker",
            Icon::SpeakerOff => "voicechat:gui/icons/speaker_off",
            Icon::Whisper => "voicechat:gui/icons/whisper",
            Icon::Disconnected => "voicechat:gui/icons/disconnected",
            Icon::Group => "voicechat:gui/icons/group",
            Icon::Settings => "voicechat:gui/icons/settings",
            Icon::Lock => "voicechat:gui/icons/lock",
            Icon::Volumes => "voicechat:gui/icons/volumes",
            Icon::Eye => "voicechat:gui/icons/eye",
            Icon::EyeOff => "voicechat:gui/icons/eye_off",
            Icon::Leave => "voicechat:gui/icons/leave",
            Icon::GroupHud => "voicechat:gui/icons/group_hud",
            Icon::GroupHudOff => "voicechat:gui/icons/group_hud_off",
            Icon::Check => "voicechat:gui/icons/check",
            Icon::Search => "voicechat:gui/icons/search",
        }
    }

    pub fn from_sprite(name: &str) -> Option<Icon> {
        Icon::ALL.into_iter().find(|i| i.sprite() == name)
    }
}

/// The whole of icon `i` drawn into `dst`.
pub fn icon(i: Icon, dst: [f32; 4]) -> Draw {
    sprite(i.sprite(), [0.0, 0.0, 16.0, 16.0], dst)
}

/// What to draw where the atlas has no sprite `d` names: the mod's icons
/// pixel by pixel (they are compiled in too), anything else as
/// [`fallback`]. Other draws stay as they are.
pub fn fallback_all(d: &Draw) -> Vec<Draw> {
    let Draw::Sprite { name, src, dst } = d else {
        return vec![d.clone()];
    };
    match Icon::from_sprite(name) {
        Some(i) => icons::fills(i, *src, *dst),
        None => vec![fallback(d)],
    }
}

/// A sprite as a plain fill at the same place, in a button's grey.
pub fn fallback(d: &Draw) -> Draw {
    match d {
        Draw::Sprite { dst, name, .. } => {
            let argb = if name.contains("highlighted") {
                0xFF8A_8A8A
            } else if name.contains("disabled") {
                0xFF3A_3A3A
            } else if name.contains("text_field") {
                0xFF00_0000
            } else {
                0xFF6F_6F6F
            };
            Draw::Fill(*dst, argb)
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_slice_makes_nine_parts_that_cover_the_rect() {
        let s = NineSlice::even("widget/button", [200.0, 20.0], 3.0);
        let d = nine_slice(&s, [10.0, 10.0, 120.0, 20.0]);
        assert_eq!(d.len(), 9);
        // Corners 3×3 in the sprite and on screen; the centre 114×14 on screen.
        assert!(matches!(&d[0], Draw::Sprite { src, dst, .. } if *src == [0.0, 0.0, 3.0, 3.0] && *dst == [10.0, 10.0, 3.0, 3.0]));
        assert!(matches!(&d[4], Draw::Sprite { dst, .. } if *dst == [13.0, 13.0, 114.0, 14.0]));
        // The parts cover the rectangle exactly.
        let area: f32 = d.iter().map(|x| if let Draw::Sprite { dst, .. } = x { dst[2] * dst[3] } else { 0.0 }).sum();
        assert_eq!(area, 120.0 * 20.0);
    }

    #[test]
    fn a_wide_button_tiles_its_centre() {
        let d = nine_slice(&BUTTON, [0.0, 0.0, 400.0, 20.0]);
        let area: f32 = d.iter().map(|x| if let Draw::Sprite { dst, src, .. } = x {
            assert!(src[2] <= 200.0 && src[3] <= 20.0);
            dst[2] * dst[3]
        } else { 0.0 }).sum();
        assert_eq!(area, 400.0 * 20.0);
        assert!(d.len() > 9);
    }

    #[test]
    fn button_state_picks_the_vanilla_sprite() {
        let hover = button([0.0, 0.0, 100.0, 20.0], ButtonState::Hover, "Settings", 40.0);
        assert!(hover.iter().any(|d| matches!(d, Draw::Sprite { name, .. } if name.ends_with("button_highlighted"))));
        assert!(hover.iter().any(|d| matches!(d, Draw::Text(t, [x, _], _, true) if t == "Settings" && *x == 30.0)));
        let off = button([0.0, 0.0, 100.0, 20.0], ButtonState::Disabled, "Off", 18.0);
        assert!(off.iter().any(|d| matches!(d, Draw::Text(_, _, rgb, _) if *rgb == LABEL_DISABLED)));
    }

    #[test]
    fn slider_handle_sits_at_the_value() {
        let d = slider([0.0, 0.0, 200.0, 20.0], 0.5, false, "Volume: 100%", 60.0);
        // An 8 px handle: (200-8)*0.5 = 96.
        assert!(d.iter().any(|x| matches!(x, Draw::Sprite { name, dst, .. } if name.ends_with("slider_handle") && dst[0] == 96.0)));
        assert_eq!(slider_value([0.0, 0.0, 200.0, 20.0], 4.0 + 0.75 * 192.0), 0.75);
    }

    #[test]
    fn fallback_turns_a_sprite_into_a_fill_at_the_same_place() {
        let f = fallback(&Draw::Sprite { name: "x".into(), src: [0.0; 4], dst: [1.0, 2.0, 3.0, 4.0] });
        assert!(matches!(f, Draw::Fill([1.0, 2.0, 3.0, 4.0], _)));
        // The mod's icons fall back to their own pixels, inside the same place.
        let px = fallback_all(&icon(Icon::MicOff, [0.0, 0.0, 16.0, 16.0]));
        assert!(px.len() > 4);
        assert!(px.iter().all(|d| matches!(d, Draw::Fill([x, y, w, h], _) if *x >= 0.0 && *y >= 0.0 && x + w <= 16.0 && y + h <= 16.0)));
    }

    #[test]
    fn icons_are_named_for_the_content_mod() {
        assert!(matches!(icon(Icon::MicOff, [0.0, 0.0, 16.0, 16.0]), Draw::Sprite { name, .. } if name == "voicechat:gui/icons/microphone_off"));
        assert_eq!(Icon::from_sprite("voicechat:gui/icons/group"), Some(Icon::Group));
    }

    #[test]
    fn long_labels_are_cut_with_an_ellipsis() {
        let tw = |t: &str| t.chars().count() as f32 * 6.0;
        let cut = fit(&"x".repeat(32), 60.0, &tw);
        assert!(cut.ends_with('…') && tw(&cut) <= 60.0, "{cut}");
        assert_eq!(fit("short", 60.0, &tw), "short");
    }
}
