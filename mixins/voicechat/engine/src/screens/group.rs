// The group screens, laid out as Simple Voice Chat's: `JoinGroupScreen` (the
// server's groups as entries, a lock on those with a password, the members as
// faces on the right and by name in a tooltip; a click joins; Create Group at
// the bottom), `GroupScreen` once in one (its members, a frame around whoever
// talks, a volume slider on the hovered one; microphone, voice chat, group
// HUD and leave at the bottom), `CreateGroupScreen` (195×124) and
// `EnterPasswordScreen` (195×74).
use super::{PADDING, Screen, Ui, window};
use crate::skin::{self, Icon};
use crate::text;
use crate::ui::{Action, Draw, Field, Screens, Slider, View, Widget};

/// A face on a group's entry, and how many show before "and n more".
const FACE: f32 = 12.0;
const TIP_MEMBERS: usize = 10;

pub fn list(ui: &mut Ui<'_>, state: &Screens, view: &View) {
    let f = ui.list_frame(text::JOIN_CREATE, 32.0, false, 3.0);
    let usable = view.connected && view.groups_enabled;
    if !usable || view.groups.is_empty() {
        let why = match () {
            _ if !view.connected => text::NOT_CONNECTED,
            _ if !view.groups_enabled => text::GROUPS_OFF,
            _ => text::NO_GROUPS,
        };
        f.empty(ui, why, 0xFF_FFFF);
    } else {
        let (first, shown) = window(state.scroll, view.groups.len(), f.rows);
        for (i, g) in view.groups.iter().skip(first).take(shown).enumerate() {
            let rect @ [x, y, w, h] = f.entry(i);
            let hover = ui.hover(rect);
            ui.draws.push(Draw::Fill(rect, if hover { skin::ENTRY_HOVER } else { skin::ENTRY }));
            let mut name_x = x + PADDING;
            if g.has_password {
                ui.draws.push(skin::icon(Icon::Lock, [x + PADDING, y + h / 2.0 - 8.0, 16.0, 16.0]));
                name_x += 16.0 + PADDING;
            }
            // Faces two high from the right, as many columns as the name leaves.
            let name = ui.fit(&g.name, x + w - PADDING - name_x - FACE - PADDING);
            ui.text(&name, [name_x, y + (h / 2.0 - 4.0).floor()], 0xFF_FFFF, false);
            let used = name_x - x + ui.width(&name);
            let per_row = ((w - (used + PADDING + PADDING)) / (FACE + 1.0)).floor().max(0.0) as usize;
            for (j, (member, id)) in g.members.iter().zip(g.member_ids.iter()).enumerate().take(per_row * 2) {
                let fx = x + w - FACE - PADDING - (j / 2) as f32 * (FACE + 1.0);
                let fy = y + (h / 2.0).floor() - (FACE + 1.0) + (FACE + 1.0) * (j % 2) as f32;
                ui.head([fx, fy, FACE, FACE], member, *id);
            }
            let mut tip = vec![text::group_title(&g.name, g.kind)];
            if g.members.is_empty() {
                tip.push(text::NO_MEMBERS.into());
            } else {
                tip.push(text::MEMBERS.into());
                tip.extend(g.members.iter().take(TIP_MEMBERS).map(|m| format!("  {m}")));
                if g.members.len() > TIP_MEMBERS {
                    tip.push(text::and_more(g.members.len() - TIP_MEMBERS));
                }
            }
            ui.tip(rect, tip);
            ui.widgets.push(Widget { rect, action: Action::Join(g.id) });
        }
        f.scrollbar(ui, first, shown, view.groups.len());
    }
    let create = [f.left + 7.0, f.footer_y(), 222.0, 20.0];
    ui.button(create, text::CREATE_GROUP, usable.then_some(Action::Open(Screen::CreateGroup)));
    ui.message(state, f.bottom());
}

pub fn members(ui: &mut Ui<'_>, state: &Screens, view: &View) {
    let Some(group) = view.groups.iter().find(|g| g.ours) else { return };
    let f = ui.list_frame(&text::in_group(&group.name, group.kind), 32.0, false, 3.0);
    let (first, shown) = window(state.scroll, view.members.len(), f.rows);
    for (i, m) in view.members.iter().skip(first).take(shown).enumerate() {
        let rect @ [x, y, w, h] = f.entry(i);
        ui.draws.push(Draw::Fill(rect, skin::ENTRY));
        // The outline is the entry's height less the padding; the face sits a
        // tenth inside it (`GroupEntry`: 10 units, the face 8 at 1).
        let outline = h - PADDING * 2.0;
        let unit = outline / 10.0;
        let [ox, oy] = [x + PADDING, y + PADDING];
        if m.talking && !m.disabled {
            ui.draws.extend(skin::talk_outline([ox, oy, outline, outline], unit.floor().max(1.0)));
        }
        let face = [(ox + unit).round(), (oy + unit).round(), (8.0 * unit).round(), (8.0 * unit).round()];
        ui.head(face, &m.name, m.id);
        if m.disabled {
            ui.draws.push(skin::icon(Icon::SpeakerOff, face));
        }
        let name_x = x + PADDING + outline + PADDING;
        let hover = ui.hover(rect) || ui.dragging == Some(Slider::Player(m.id));
        let slider_w = if hover && !m.own {
            (w - (PADDING + outline + PADDING + ui.width(&m.name) + PADDING + PADDING)).min(100.0).max(40.0)
        } else {
            0.0
        };
        let name = ui.fit(&m.name, w - (name_x - x) - PADDING - if slider_w > 0.0 { slider_w + PADDING } else { 0.0 });
        ui.text(&name, [name_x, y + (h / 2.0 - 4.0).floor()], 0xFF_FFFF, false);
        if slider_w > 0.0 {
            let slider = [x + w - slider_w - PADDING, y + ((h - 20.0) / 2.0).floor(), slider_w, 20.0];
            let value = super::volumes::player_value(m.volume);
            ui.slider(slider, value, &text::percent(m.volume), Slider::Player(m.id), Vec::new());
        }
    }
    f.scrollbar(ui, first, shown, view.members.len());
    let s = &view.settings;
    let at = |dx: f32| [f.left + dx, f.footer_y(), 20.0, 20.0];
    let mic = if s.muted { Icon::MicOff } else { Icon::Mic };
    ui.icon_button(at(7.0), mic, Some(Action::Mute), text::microphone_tip(s.muted));
    let speaker = if s.disabled { Icon::SpeakerOff } else { Icon::Speaker };
    ui.icon_button(at(30.0), speaker, Some(Action::Disable), text::voice_chat_tip(s.disabled));
    let hud = if s.hud { Icon::GroupHud } else { Icon::GroupHudOff };
    ui.icon_button(at(53.0), hud, Some(Action::GroupHud), text::group_hud_tip(s.hud));
    ui.icon_button(at(209.0), Icon::Leave, Some(Action::Leave), text::LEAVE_TIP.into());
    ui.message(state, f.bottom());
}

pub fn create(ui: &mut Ui<'_>, state: &Screens) {
    let [x, y, _, h] = ui.panel(195.0, 124.0, text::CREATE_GROUP);
    ui.label(text::GROUP_NAME, [x + 8.0, y + 21.0]);
    ui.field([x + 7.0, y + 30.0, 181.0, 14.0], &state.name, "", false, state.focus == Some(Field::Name), Action::Focus(Field::Name));
    ui.label(text::PASSWORD_OPTIONAL, [x + 8.0, y + 47.0]);
    let focus = state.focus == Some(Field::Password);
    ui.field([x + 7.0, y + 56.0, 181.0, 14.0], &state.password, "", true, focus, Action::Focus(Field::Password));
    let kind = [x + 6.0, y + 74.0, 183.0, 20.0];
    ui.button(kind, &text::group_type(state.kind), Some(Action::Kind));
    ui.tip(kind, vec![text::KIND_TIPS[state.kind.rem_euclid(3) as usize].into()]);
    let ready = !state.name.trim().is_empty();
    ui.button([x + 6.0, y + 97.0, 183.0, 20.0], text::CREATE, ready.then_some(Action::Create));
    ui.message(state, y + h);
}

pub fn password(ui: &mut Ui<'_>, state: &Screens, view: &View) {
    let title = state
        .joining
        .and_then(|id| view.groups.iter().find(|g| g.id == id))
        .map_or_else(|| text::ENTER_PASSWORD.to_owned(), |g| format!("{}: {}", text::ENTER_PASSWORD, g.name));
    let [x, y, _, h] = ui.panel(195.0, 74.0, &title);
    ui.label(text::PASSWORD, [x + 8.0, y + 21.0]);
    let focus = state.focus == Some(Field::JoinPassword);
    ui.field([x + 7.0, y + 30.0, 181.0, 14.0], &state.join_password, "", true, focus, Action::Focus(Field::JoinPassword));
    let ready = !state.join_password.is_empty();
    ui.button([x + 7.0, y + 47.0, 181.0, 20.0], text::JOIN_GROUP, ready.then_some(Action::JoinWithPassword));
    ui.message(state, y + h);
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::{Screen, layout};
    use super::*;
    use crate::ui::{GroupRow, MemberRow};

    fn groups(n: usize) -> View {
        let mut v = view();
        v.groups = (0..n)
            .map(|i| GroupRow {
                id: i as u128 + 1,
                name: format!("Group {i}"),
                has_password: false,
                members: vec![],
                member_ids: vec![],
                ours: false,
                kind: 0,
            })
            .collect();
        v
    }

    #[test]
    fn groups_are_entries_a_click_joins() {
        let size = [320.0, 240.0];
        let (draws, widgets) = layout(&Screen::Group, &Screens::default(), &view(), size, [0.0, 0.0], &tw);
        // Entries 216×32 from the list's top, 36 apart.
        let builders = widget(&widgets, &Action::Join(1)).unwrap();
        let secret = widget(&widgets, &Action::Join(2)).unwrap();
        assert_eq!(builders, [52.0, 52.0, 216.0, 32.0]);
        assert_eq!(secret[1] - builders[1], 36.0);
        assert_eq!(text_at(&draws, "Builders").unwrap(), [56.0, 64.0]);
        // The lock pushes the name right.
        assert_eq!(text_at(&draws, "Secret").unwrap()[0], 56.0 + 20.0);
        assert!(draws.iter().any(|d| matches!(d, Draw::Sprite { name, .. } if name == Icon::Lock.sprite())));
        assert!(widget(&widgets, &Action::Open(Screen::CreateGroup)).unwrap()[2] == 222.0);
        assert!(texts(&draws).contains(&"Join/Create Group".to_owned()));
        // The tooltip names the members.
        let (draws, _) = layout(&Screen::Group, &Screens::default(), &view(), size, centre(builders), &tw);
        let t = texts(&draws);
        assert!(t.contains(&"Members:".to_owned()) && t.contains(&"  Alex".to_owned()), "{t:?}");
        assert!(on_screen(&widgets, size));
    }

    #[test]
    fn a_long_name_is_cut_inside_its_entry() {
        let size = [320.0, 240.0];
        let mut v = view();
        v.groups[0].name = "A".repeat(60);
        let (draws, widgets) = layout(&Screen::Group, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        let cut = texts(&draws).into_iter().find(|t| t.starts_with("AAA")).unwrap();
        assert!(cut.ends_with('…') && cut.chars().count() < 60, "{cut}");
        let at = text_at(&draws, &cut).unwrap();
        let entry = widget(&widgets, &Action::Join(1)).unwrap();
        assert!(at[0] + tw(&cut) <= entry[0] + entry[2] - 4.0);
    }

    #[test]
    fn many_groups_scroll_with_the_wheel() {
        let size = [320.0, 240.0];
        let v = groups(25);
        let joins = |state: &Screens| {
            let (_, widgets) = layout(&Screen::Group, state, &v, size, [0.0, 0.0], &tw);
            widgets.iter().filter_map(|w| if let Action::Join(id) = w.action { Some(id) } else { None }).collect::<Vec<_>>()
        };
        let mut state = Screens::default();
        assert_eq!(joins(&state).len(), 3);
        for _ in 0..3 {
            state.scroll_by(-1);
        }
        assert_eq!(joins(&state)[0], 4);
        for _ in 0..50 {
            state.scroll_by(-1);
        }
        assert_eq!(*joins(&state).last().unwrap(), 25);
    }

    #[test]
    fn in_a_group_its_members_and_the_buttons_under_them() {
        let size = [320.0, 240.0];
        let mut v = view();
        v.groups[0].ours = true;
        v.groups[0].kind = 2;
        v.members = vec![
            MemberRow { id: 0xA, name: "Alex".into(), talking: true, disabled: false, own: false, volume: 100 },
            MemberRow { id: 0xC, name: "Me".into(), talking: false, disabled: true, own: true, volume: 100 },
        ];
        let (draws, widgets) = layout(&Screen::Group, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        assert!(texts(&draws).contains(&"Group: Builders (Isolated)".to_owned()));
        // The name after the 24 px outline.
        assert_eq!(text_at(&draws, "Alex").unwrap(), [52.0 + 4.0 + 24.0 + 4.0, 64.0]);
        // A frame for the one talking, a speaker off over the one who is off.
        assert!(draws.iter().any(|d| matches!(d, Draw::Fill(_, 0xFF55_FF55))));
        assert!(draws.iter().any(|d| matches!(d, Draw::Sprite { name, .. } if name == Icon::SpeakerOff.sprite())));
        let rel = |a: &Action| widget(&widgets, a).unwrap()[0] - 44.0;
        assert_eq!([rel(&Action::Mute), rel(&Action::Disable), rel(&Action::GroupHud), rel(&Action::Leave)], [7.0, 30.0, 53.0, 209.0]);
        // Hovering another member shows a volume slider; not on yourself.
        let slider = |mouse| {
            let (_, w) = layout(&Screen::Group, &Screens::default(), &v, size, mouse, &tw);
            w.iter().find_map(|w| if let Action::Slide(Slider::Player(id), _) = w.action { Some((id, w.rect)) } else { None })
        };
        let (id, r) = slider([60.0, 60.0]).unwrap();
        assert_eq!((id, r[2], r[0] + r[2]), (0xA, 100.0, 52.0 + 216.0 - 4.0));
        assert!(slider([60.0, 96.0]).is_none());
        assert!(on_screen(&widgets, size));
    }

    #[test]
    fn the_create_screen_as_simple_voice_chats() {
        let size = [320.0, 240.0];
        let (draws, widgets) = layout(&Screen::CreateGroup, &Screens::default(), &view(), size, [0.0, 0.0], &tw);
        let (left, top) = (((320.0 - 195.0) / 2.0_f32).floor(), ((240.0 - 124.0) / 2.0_f32).floor());
        let rel = |a: &Action| {
            let r = widget(&widgets, a).unwrap();
            [r[0] - left, r[1] - top, r[2], r[3]]
        };
        assert_eq!(rel(&Action::Focus(Field::Name)), [7.0, 30.0, 181.0, 14.0]);
        assert_eq!(rel(&Action::Focus(Field::Password)), [7.0, 56.0, 181.0, 14.0]);
        assert_eq!(rel(&Action::Kind), [6.0, 74.0, 183.0, 20.0]);
        // Create waits for a name.
        assert!(widget(&widgets, &Action::Create).is_none());
        assert!(texts(&draws).contains(&"Group Type: Normal".to_owned()));
        let mut state = Screens::default();
        state.name = "Team".into();
        let (_, widgets) = layout(&Screen::CreateGroup, &state, &view(), size, [0.0, 0.0], &tw);
        assert_eq!(rel_of(&widgets, &Action::Create, left, top), [6.0, 97.0, 183.0, 20.0]);
    }

    fn rel_of(widgets: &[Widget], a: &Action, left: f32, top: f32) -> [f32; 4] {
        let r = widget(widgets, a).unwrap();
        [r[0] - left, r[1] - top, r[2], r[3]]
    }

    #[test]
    fn the_password_screen_joins_once_something_is_typed() {
        let size = [320.0, 240.0];
        let mut state = Screens::default();
        state.joining = Some(2);
        state.focus = Some(Field::JoinPassword);
        let (draws, widgets) = layout(&Screen::Password, &state, &view(), size, [0.0, 0.0], &tw);
        assert!(texts(&draws).contains(&"Enter Password: Secret".to_owned()));
        let (left, top) = (((320.0 - 195.0) / 2.0_f32).floor(), ((240.0 - 74.0) / 2.0_f32).floor());
        assert_eq!(rel_of(&widgets, &Action::Focus(Field::JoinPassword), left, top), [7.0, 30.0, 181.0, 14.0]);
        assert!(widget(&widgets, &Action::JoinWithPassword).is_none());
        state.join_password = "pw".into();
        let (_, widgets) = layout(&Screen::Password, &state, &view(), size, [0.0, 0.0], &tw);
        assert_eq!(rel_of(&widgets, &Action::JoinWithPassword, left, top), [7.0, 47.0, 181.0, 20.0]);
    }
}
