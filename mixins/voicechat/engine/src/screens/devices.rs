// Choosing the microphone or the speakers, laid out as Simple Voice Chat's
// `SelectDeviceScreen`: Default (the system's), then each device by name, as
// entries with the device's icon, the chosen one darker with a check.
use super::{DeviceKind, PADDING, Ui, window};
use crate::skin::{self, Icon};
use crate::text;
use crate::ui::{Action, Draw, Screens, View, Widget};

pub fn layout(ui: &mut Ui<'_>, state: &Screens, view: &View, kind: DeviceKind) {
    let title = match kind {
        DeviceKind::Input => text::SELECT_MICROPHONE,
        DeviceKind::Output => text::SELECT_SPEAKER,
    };
    let f = ui.list_frame(title, 32.0, false, 3.0);
    let chosen = match kind {
        DeviceKind::Input => view.settings.mic_device.as_deref(),
        DeviceKind::Output => view.settings.speaker_device.as_deref(),
    };
    // A device chosen before but gone now is not listed: Default is in use.
    let chosen = chosen.filter(|c| view.devices.iter().any(|d| d == c));
    let icon = match kind {
        DeviceKind::Input => Icon::Mic,
        DeviceKind::Output => Icon::Speaker,
    };
    let entries: Vec<Option<&str>> = std::iter::once(None).chain(view.devices.iter().map(|d| Some(d.as_str()))).collect();
    let (first, shown) = window(state.scroll, entries.len(), f.rows);
    for (i, entry) in entries.iter().skip(first).take(shown).enumerate() {
        let rect @ [x, y, w, h] = f.entry(i);
        let selected = *entry == chosen;
        let bg = match () {
            _ if selected => skin::ENTRY_CHOSEN,
            _ if ui.hover(rect) => skin::ENTRY_HOVER,
            _ => skin::ENTRY,
        };
        ui.draws.push(Draw::Fill(rect, bg));
        let at = [x + PADDING, y + (h / 2.0).floor() - 8.0, 16.0, 16.0];
        ui.draws.push(skin::icon(if selected { Icon::Check } else { icon }, at));
        let label = entry.unwrap_or(text::DEFAULT_DEVICE);
        let shown_label = ui.fit(label, w - PADDING - 16.0 - PADDING - PADDING);
        ui.text(&shown_label, [x + PADDING + 16.0 + PADDING, y + (h / 2.0 - 4.0).floor()], 0xFF_FFFF, false);
        if shown_label != label {
            ui.tip(rect, vec![label.to_owned()]);
        }
        ui.widgets.push(Widget { rect, action: Action::Device(kind, entry.map(str::to_owned)) });
    }
    f.scrollbar(ui, first, shown, entries.len());
    if view.devices.is_empty() && f.rows > 1 {
        let e = f.entry(1);
        ui.centred(text::NO_DEVICES, (ui.size[0] / 2.0).floor(), e[1] + 12.0, 0xA0_A0A0, true);
    }
    ui.back([f.left + 7.0, f.footer_y(), 222.0, 20.0], text::BACK);
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::{Screen, layout};
    use super::*;

    /// The background of the entry whose label is `want`.
    fn colour(draws: &[Draw], want: &str) -> u32 {
        let at = text_at(draws, want).unwrap();
        draws
            .iter()
            .rev()
            .find_map(|d| match d {
                Draw::Fill([x, y, w, h], argb) if *w == 216.0 && *h == 32.0 && at[0] > *x && at[1] > *y && at[1] < y + h => {
                    Some(*argb)
                }
                _ => None,
            })
            .unwrap()
    }

    #[test]
    fn default_first_and_the_chosen_one_green() {
        let size = [320.0, 240.0];
        let mut v = view();
        v.devices = vec!["USB Mic".into(), "Headset".into()];
        let screen = Screen::Devices(DeviceKind::Input);
        let (draws, widgets) = layout(&screen, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        let rows: Vec<&Action> = widgets.iter().map(|w| &w.action).filter(|a| matches!(a, Action::Device(..))).collect();
        assert_eq!(rows[0], &Action::Device(DeviceKind::Input, None));
        assert_eq!(rows[2], &Action::Device(DeviceKind::Input, Some("Headset".into())));
        assert_eq!(colour(&draws, "Default"), skin::ENTRY_CHOSEN);
        v.settings.mic_device = Some("Headset".into());
        let (draws, _) = layout(&screen, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        assert_eq!((colour(&draws, "Default"), colour(&draws, "Headset")), (skin::ENTRY, skin::ENTRY_CHOSEN));
        let checks = draws.iter().filter(|d| matches!(d, Draw::Sprite { name, .. } if name == Icon::Check.sprite())).count();
        assert_eq!(checks, 1);
        // A device that is gone: Default again.
        v.settings.mic_device = Some("Unplugged".into());
        let (draws, _) = layout(&screen, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        assert_eq!(colour(&draws, "Default"), skin::ENTRY_CHOSEN);
        assert!(on_screen(&widgets, size));
    }
}
