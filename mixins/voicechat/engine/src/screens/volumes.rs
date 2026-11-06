// Adjust Volumes, laid out as Simple Voice Chat's `AdjustVolumesScreen`: a
// Players tab over the panel, a search box under the title, and an entry for
// each other player with voice chat (face, name, a 100 px slider from 0 to
// 400 %), kept per player.
use super::{ENTRY_HEIGHT, LIST_WIDTH, PADDING, Ui, window};
use crate::skin::{self, Icon};
use crate::text;
use crate::ui::{Action, Draw, Field, Screens, Slider, View, Widget};

pub const PLAYER_MAX: f32 = 400.0;
const FACE: f32 = 24.0;
const SLIDER: f32 = 100.0;

pub fn player_value(percent: u32) -> f32 {
    percent as f32 / PLAYER_MAX
}

pub fn player_of(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * PLAYER_MAX).round() as u32
}

pub fn layout(ui: &mut Ui<'_>, state: &Screens, view: &View) {
    let f = ui.list_frame(text::VOLUMES, 8.0, true, 4.0);
    // The one tab, chosen (so not clickable), as the Java mod shows it.
    ui.button([f.left, f.top - 22.0, LIST_WIDTH, 20.0], text::PLAYERS_TAB, None);
    ui.draws.push(skin::icon(Icon::Search, [f.left + 10.0, f.top + 20.0, 12.0, 12.0]));
    let search = [f.left + 28.0, f.top + 22.0, 196.0, 12.0];
    let focused = state.focus == Some(Field::Search);
    if state.search.is_empty() && !focused {
        ui.text(text::SEARCH_HINT, [search[0], search[1] + 1.0], 0x80_8080, false);
    } else {
        let mut shown = state.search.clone();
        while !shown.is_empty() && ui.width(&shown) > search[2] - 8.0 {
            shown.remove(0);
        }
        let caret = if focused { "_" } else { "" };
        ui.text(&format!("{shown}{caret}"), [search[0], search[1] + 1.0], 0xFF_FFFF, true);
    }
    ui.widgets.push(Widget { rect: search, action: Action::Focus(Field::Search) });

    let needle = state.search.to_lowercase();
    let players: Vec<_> = view.players.iter().filter(|p| p.name.to_lowercase().contains(&needle)).collect();
    if players.is_empty() {
        let why = if view.players.is_empty() { text::NO_PLAYERS } else { text::NO_MATCH };
        f.empty(ui, why, 0xA0_A0A0);
        return;
    }
    let (first, shown) = window(state.scroll, players.len(), f.rows);
    for (i, p) in players.iter().skip(first).take(shown).enumerate() {
        let rect @ [x, y, w, h] = f.entry(i);
        ui.draws.push(Draw::Fill(rect, skin::ENTRY));
        ui.head([x + PADDING, y + ((h - FACE) / 2.0).floor(), FACE, FACE], &p.name, p.id);
        let name_x = x + PADDING + FACE + PADDING;
        let room = w - PADDING - FACE - PADDING - PADDING - SLIDER - PADDING;
        let name = ui.fit(&p.name, room);
        ui.text(&name, [name_x, y + (h / 2.0 - 4.0).floor()], 0xFF_FFFF, false);
        let slider = [x + w - SLIDER - PADDING, y + ((ENTRY_HEIGHT - 20.0) / 2.0).floor(), SLIDER, 20.0];
        ui.slider(slider, player_value(p.volume), &text::percent(p.volume), Slider::Player(p.id), Vec::new());
    }
    f.scrollbar(ui, first, shown, players.len());
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::{Screen, layout};
    use super::*;
    use crate::ui::{Action, PlayerRow};

    #[test]
    fn a_slider_per_player_from_0_to_400() {
        let size = [320.0, 240.0];
        let mut v = view();
        v.players = vec![
            PlayerRow { id: 0xA, name: "Alex".into(), volume: 100 },
            PlayerRow { id: 0xB, name: "Bea".into(), volume: 200 },
        ];
        let (draws, widgets) = layout(&Screen::Volumes, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        assert!(texts(&draws).contains(&"200%".to_owned()) && texts(&draws).contains(&"Bea".to_owned()));
        let r = widgets.iter().find(|w| matches!(w.action, Action::Slide(Slider::Player(0xB), _))).unwrap().rect;
        let mut m = Screens::default();
        m.set_widgets(widgets.clone());
        let Some(Action::Slide(Slider::Player(id), value)) = m.press([r[0] + 4.0 + 0.25 * (r[2] - 8.0), r[1] + 5.0]) else {
            panic!()
        };
        assert_eq!((id, player_of(value)), (0xB, 100));
        // 100 px, at the entry's right.
        assert_eq!((r[2], r[0] + r[2]), (100.0, 52.0 + 216.0 - 4.0));
        assert!(on_screen(&widgets, size));
        assert!(texts(&draws).contains(&"Players".to_owned()) && texts(&draws).contains(&"Search...".to_owned()));
    }

    #[test]
    fn the_search_box_keeps_the_matching_players() {
        let size = [320.0, 240.0];
        let mut v = view();
        v.players = vec![
            PlayerRow { id: 0xA, name: "Alex".into(), volume: 100 },
            PlayerRow { id: 0xB, name: "Bea".into(), volume: 200 },
        ];
        let mut state = Screens::default();
        state.search = "BE".into();
        let (draws, widgets) = layout(&Screen::Volumes, &state, &v, size, [0.0, 0.0], &tw);
        let ids: Vec<u128> =
            widgets.iter().filter_map(|w| if let Action::Slide(Slider::Player(id), _) = w.action { Some(id) } else { None }).collect();
        assert_eq!(ids, vec![0xB]);
        assert!(texts(&draws).contains(&"BE".to_owned()));
        state.search = "zz".into();
        let (draws, _) = layout(&Screen::Volumes, &state, &v, size, [0.0, 0.0], &tw);
        assert!(texts(&draws).contains(&"No players found".to_owned()));
    }
}
