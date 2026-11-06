// The main screen (V), laid out as Simple Voice Chat's `VoiceChatScreen`
// (195×76): Settings and Group side by side under the title, and a row of
// icon buttons at the bottom (microphone, voice chat, Adjust Volumes on the
// left, hide icons on the right). While voice chat isn't connected, why
// shows under the panel.
use super::{Screen, Ui};
use crate::skin::Icon;
use crate::text;
use crate::ui::{Action, View};

pub const WIDTH: f32 = 195.0;
pub const HEIGHT: f32 = 76.0;

pub fn layout(ui: &mut Ui<'_>, view: &View) {
    let [x, y, _, h] = ui.panel(WIDTH, HEIGHT, text::TITLE);
    let s = &view.settings;
    ui.button([x + 6.0, y + 21.0, 75.0, 20.0], text::SETTINGS_BUTTON, Some(Action::Open(Screen::Settings)));
    let group = [x + 114.0, y + 21.0, 75.0, 20.0];
    let usable = view.connected && view.groups_enabled;
    ui.button(group, text::GROUP_BUTTON, usable.then_some(Action::Open(Screen::Group)));
    if !usable {
        let why = if view.connected { text::GROUPS_OFF } else { text::NOT_CONNECTED };
        ui.tip(group, vec![why.into()]);
    }
    let at = |dx: f32| [x + dx, y + 50.0, 20.0, 20.0];
    let mic = if s.muted { Icon::MicOff } else { Icon::Mic };
    ui.icon_button(at(6.0), mic, Some(Action::Mute), text::microphone_tip(s.muted));
    let speaker = if s.disabled { Icon::SpeakerOff } else { Icon::Speaker };
    ui.icon_button(at(28.0), speaker, Some(Action::Disable), text::voice_chat_tip(s.disabled));
    ui.icon_button(at(50.0), Icon::Volumes, Some(Action::Open(Screen::Volumes)), text::VOLUMES_TIP.into());
    let eye = if s.hide_icons { Icon::EyeOff } else { Icon::Eye };
    ui.icon_button(at(169.0), eye, Some(Action::HideIcons), text::hide_icons_tip(s.hide_icons));
    if !view.connected {
        let (status, rgb) = &view.status;
        let status = ui.fit(status, ui.size[0] - 8.0);
        if y + h + 14.0 <= ui.size[1] {
            ui.centred(&status, (ui.size[0] / 2.0).floor(), y + h + 6.0, *rgb, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::{Screen, layout};
    use crate::ui::{Action, Screens};

    #[test]
    fn buttons_where_simple_voice_chat_puts_them_at_every_gui_size() {
        for size in [[320.0, 240.0], [960.0, 540.0], [427.0, 240.0]] {
            let (draws, widgets) = layout(&Screen::Main, &Screens::default(), &view(), size, [0.0, 0.0], &tw);
            let (left, top) = (((size[0] - 195.0) / 2.0_f32).floor(), ((size[1] - 76.0) / 2.0_f32).floor());
            let title = text_at(&draws, "Voice Chat").unwrap();
            assert!((title[0] + tw("Voice Chat") / 2.0 - (left + 97.5)).abs() <= 1.0 && title[1] == top + 7.0, "{size:?}");
            let at = |a: &Action| {
                let r = widget(&widgets, a).unwrap();
                [r[0] - left, r[1] - top, r[2], r[3]]
            };
            assert_eq!(at(&Action::Open(Screen::Settings)), [6.0, 21.0, 75.0, 20.0]);
            assert_eq!(at(&Action::Open(Screen::Group)), [114.0, 21.0, 75.0, 20.0]);
            assert_eq!(at(&Action::Mute), [6.0, 50.0, 20.0, 20.0]);
            assert_eq!(at(&Action::Disable), [28.0, 50.0, 20.0, 20.0]);
            assert_eq!(at(&Action::Open(Screen::Volumes)), [50.0, 50.0, 20.0, 20.0]);
            assert_eq!(at(&Action::HideIcons), [169.0, 50.0, 20.0, 20.0]);
            assert!(on_screen(&widgets, size));
            // Connected: no status line under the panel.
            assert!(!texts(&draws).contains(&"Connected".to_owned()));
        }
    }

    #[test]
    fn groups_need_a_connection_and_a_tooltip_says_so() {
        let mut v = view();
        v.connected = false;
        v.status = ("Connecting...".into(), 0xFFFF55);
        let size = [320.0, 240.0];
        let (draws, widgets) = layout(&Screen::Main, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        assert!(widget(&widgets, &Action::Open(Screen::Group)).is_none());
        assert!(texts(&draws).contains(&"Connecting...".to_owned()));
        let group = text_at(&draws, "Group").unwrap();
        let (draws, _) = layout(&Screen::Main, &Screens::default(), &v, size, [group[0] + 1.0, group[1] + 1.0], &tw);
        assert!(texts(&draws).contains(&"Voice chat isn't connected".to_owned()));
        // Hovering the mute button names the microphone's state.
        let (_, widgets) = layout(&Screen::Main, &Screens::default(), &view(), size, [0.0, 0.0], &tw);
        let mute = centre(widget(&widgets, &Action::Mute).unwrap());
        let (draws, _) = layout(&Screen::Main, &Screens::default(), &view(), size, mute, &tw);
        assert!(texts(&draws).contains(&"Microphone: On".to_owned()));
    }
}
