//! What the voice chat draws over the world, as Simple Voice Chat's HUD
//! does: one status icon near the bottom left corner, the group's members in
//! the top left with a speaker by whoever is talking, and problems (no
//! microphone, no libopus) as red lines.
use crate::skin::{self, Icon};
use crate::ui::Draw;

/// The voice chat as the status icon shows it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HudState {
    /// The voice connection is up.
    pub connected: bool,
    pub muted: bool,
    /// Voice chat turned off: nothing heard, nothing sent.
    pub disabled: bool,
    /// The microphone is being sent now.
    pub transmitting: bool,
    pub whispering: bool,
}

/// The icon's size, and where Simple Voice Chat puts it (`hud_icon_pos_x`
/// 16, `hud_icon_pos_y` -16): 16 px from the left edge, its bottom 16 px
/// above the screen's.
pub const ICON: f32 = 16.0;
pub const ICON_X: f32 = 16.0;
pub const ICON_BOTTOM: f32 = 16.0;
/// The group list's and the problem lines' distance from the edges.
const MARGIN: f32 = 4.0;

/// The status icon in the bottom left of a `size` screen, in Simple Voice
/// Chat's order (disconnected, off, muted, whispering, talking): none while
/// nothing is worth saying (connected, quiet, not muted).
pub fn status_icon(state: &HudState, size: [f32; 2]) -> Vec<Draw> {
    let icon = if !state.connected {
        Icon::Disconnected
    } else if state.disabled {
        Icon::SpeakerOff
    } else if state.muted {
        Icon::MicOff
    } else if state.transmitting && state.whispering {
        Icon::Whisper
    } else if state.transmitting {
        Icon::Mic
    } else {
        return Vec::new();
    };
    vec![skin::icon(icon, [ICON_X, size[1] - ICON_BOTTOM - ICON, ICON, ICON])]
}

/// One member of the group list.
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub id: u128,
    pub name: String,
    pub talking: bool,
    /// Voice chat turned off.
    pub disabled: bool,
}

/// The group list as Simple Voice Chat draws it (`GroupChatManager`, at
/// scale 2): one 20 px cell per member, 22 px apart downwards, a face (16 px,
/// 2 px in) framed while they talk and a small speaker off on it while their
/// voice chat is off. The game gives mods no skins, so the face is a stand-in
/// and the name follows it.
const CELL: f32 = 20.0;
const STEP: f32 = 22.0;

/// The group's members from the top left (the list's own size is
/// [`group_list_size`]).
pub fn group_list(members: &[Member], _size: [f32; 2], text_width: &dyn Fn(&str) -> f32) -> Vec<Draw> {
    let mut out = Vec::new();
    for (i, m) in members.iter().enumerate() {
        let (x, y) = (MARGIN, MARGIN + i as f32 * STEP);
        if m.talking && !m.disabled {
            out.extend(skin::talk_outline([x, y, CELL, CELL], 2.0));
        }
        let face = [x + 2.0, y + 2.0, 16.0, 16.0];
        out.extend(skin::head(face, &m.name, m.id, text_width));
        if m.disabled {
            out.push(skin::icon(Icon::SpeakerOff, [x + 10.0, y + 10.0, 10.0, 10.0]));
        }
        out.push(Draw::Text(m.name.clone(), [x + CELL + 3.0, y + 6.0], 0xFF_FFFF, true));
    }
    out
}

/// The width and height the group list takes.
pub fn group_list_size(members: &[Member], text_width: &dyn Fn(&str) -> f32) -> [f32; 2] {
    if members.is_empty() {
        return [0.0, 0.0];
    }
    let widest = members.iter().map(|m| text_width(&m.name)).fold(0.0, f32::max);
    [CELL + 3.0 + widest + 2.0, members.len() as f32 * STEP - 2.0]
}

/// Room around the problem lines.
const ROW: f32 = 12.0;

/// Red lines for what doesn't work, above the status icon.
pub fn problems(lines: &[String], size: [f32; 2], text_width: &dyn Fn(&str) -> f32) -> Vec<Draw> {
    let mut out = Vec::new();
    let bottom = size[1] - ICON_BOTTOM - ICON - 4.0;
    for (i, line) in lines.iter().rev().enumerate() {
        let y = bottom - (i as f32 + 1.0) * ROW;
        let w = text_width(line) + 4.0;
        out.push(Draw::Fill([MARGIN - 2.0, y, w, ROW], 0x9000_0000));
        out.push(Draw::Text(line.clone(), [MARGIN, y + 2.0], 0xFF_5555, true));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn only_icon(d: &[Draw]) -> Option<(String, [f32; 4])> {
        match d {
            [Draw::Sprite { name, dst, .. }] => Some((name.clone(), *dst)),
            [] => None,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_status_icon_says_what_matters() {
        let size = [320.0, 240.0];
        let on = HudState { connected: true, ..HudState::default() };
        let talking = HudState { transmitting: true, ..on };
        assert_eq!(only_icon(&status_icon(&talking, size)), Some((Icon::Mic.sprite().into(), [16.0, 208.0, 16.0, 16.0])));
        let whisper = HudState { whispering: true, ..talking };
        assert_eq!(only_icon(&status_icon(&whisper, size)).unwrap().0, Icon::Whisper.sprite());
        let muted = HudState { muted: true, ..on };
        assert_eq!(only_icon(&status_icon(&muted, size)).unwrap().0, Icon::MicOff.sprite());
        let off = HudState { disabled: true, ..on };
        assert_eq!(only_icon(&status_icon(&off, size)).unwrap().0, Icon::SpeakerOff.sprite());
        let lost = HudState::default();
        assert_eq!(only_icon(&status_icon(&lost, size)).unwrap().0, Icon::Disconnected.sprite());
        // Disconnected comes before off, as in Simple Voice Chat.
        let lost_off = HudState { disabled: true, ..lost };
        assert_eq!(only_icon(&status_icon(&lost_off, size)).unwrap().0, Icon::Disconnected.sprite());
        assert_eq!(only_icon(&status_icon(&on, size)), None);
        // It follows the screen's bottom edge.
        assert_eq!(only_icon(&status_icon(&talking, [960.0, 540.0])).unwrap().1, [16.0, 508.0, 16.0, 16.0]);
    }

    #[test]
    fn the_group_list_frames_who_talks() {
        let tw = |t: &str| t.len() as f32 * 6.0;
        let m = |id, name: &str, talking, disabled| Member { id, name: name.into(), talking, disabled };
        let members = vec![m(1, "Alex", false, false), m(2, "Bea", true, false), m(3, "Cem", false, true)];
        let d = group_list(&members, [320.0, 240.0], &tw);
        let names: Vec<(String, [f32; 2])> = d
            .iter()
            .filter_map(|x| if let Draw::Text(t, at, ..) = x { (t.len() > 1).then(|| (t.clone(), *at)) } else { None })
            .collect();
        assert_eq!(names.iter().map(|n| n.0.as_str()).collect::<Vec<_>>(), ["Alex", "Bea", "Cem"]);
        // 22 px apart, from (4, 4); the name after the 20 px cell.
        assert_eq!(names[0].1, [27.0, 10.0]);
        assert_eq!(names[1].1[1] - names[0].1[1], 22.0);
        // The frame only around Bea's cell.
        let frames: Vec<[f32; 4]> =
            d.iter().filter_map(|x| if let Draw::Fill(r, 0xFF55_FF55) = x { Some(*r) } else { None }).collect();
        assert_eq!(frames[0], [4.0, 26.0, 20.0, 2.0]);
        assert!(frames.iter().all(|r| r[1] >= 26.0 && r[1] < 48.0));
        let off = d.iter().filter(|x| matches!(x, Draw::Sprite { name, .. } if name == Icon::SpeakerOff.sprite())).count();
        assert_eq!(off, 1);
        assert_eq!(group_list_size(&members, &tw), [20.0 + 3.0 + 24.0 + 2.0, 64.0]);
        assert!(group_list(&[], [320.0, 240.0], &tw).is_empty());
    }
}
