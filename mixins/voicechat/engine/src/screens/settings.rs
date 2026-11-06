// The settings screen, laid out as Simple Voice Chat's
// `VoiceChatSettingsScreen` (248×219, one column of 228 wide widgets 21 px
// apart): volume, amplification, noise suppression, the activation type
// with the microphone test beside it, the push to talk key, the activation
// threshold with a live level meter, group names and protocol (where the Java
// mod has its audio type), and the two device lists.
use super::{DeviceKind, Screen, Ui};
use crate::net::Mode;
use crate::skin::Icon;
use crate::text;
use crate::ui::{Action, Draw, Slider, View};

pub const WIDTH: f32 = 248.0;
pub const HEIGHT: f32 = 219.0;

pub const VOLUME_MAX: f32 = 200.0;
pub const GAIN_MAX: f32 = 500.0;
pub const THRESHOLD_MIN: f32 = -127.0;

/// A slider's place (0..=1) for a value, and back.
pub fn volume_value(percent: u32) -> f32 {
    percent as f32 / VOLUME_MAX
}
pub fn gain_value(percent: u32) -> f32 {
    percent as f32 / GAIN_MAX
}
pub fn threshold_value(db: i32) -> f32 {
    (db as f32 - THRESHOLD_MIN) / -THRESHOLD_MIN
}
pub fn volume_of(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * VOLUME_MAX).round() as u32
}
pub fn gain_of(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * GAIN_MAX).round() as u32
}
pub fn threshold_of(v: f32) -> i32 {
    (THRESHOLD_MIN + v.clamp(0.0, 1.0) * -THRESHOLD_MIN) as i32
}

/// The microphone's level (`level_db`, -127 to 0) inside a slider's track
/// at `rect`: bright past the threshold, as Simple Voice Chat's meter.
pub fn level_meter(rect: [f32; 4], level_db: f32, threshold_db: i32) -> Vec<Draw> {
    let [x, y, w, h] = rect;
    let part = ((level_db.clamp(THRESHOLD_MIN, 0.0) - THRESHOLD_MIN) / -THRESHOLD_MIN) * (w - 2.0);
    let loud = level_db > threshold_db as f32;
    vec![Draw::Fill([x + 1.0, y + 1.0, part, h - 2.0], if loud { 0xC040_C040 } else { 0x8030_6030 })]
}

pub fn layout(ui: &mut Ui<'_>, view: &View) {
    let s = &view.settings;
    let [px, py, _, _] = ui.panel(WIDTH, HEIGHT, text::SETTINGS);
    let (x, w) = (px + 10.0, WIDTH - 20.0);
    let row = |i: f32| py + 20.0 + i * 21.0;
    let half = ((WIDTH - 20.0) / 2.0).floor() - 1.0;
    let right = px + (WIDTH / 2.0).floor() + 1.0;

    ui.slider([x, row(0.0), w, 20.0], volume_value(s.volume), &text::volume(s.volume), Slider::Volume, Vec::new());
    ui.slider([x, row(1.0), w, 20.0], gain_value(s.mic_gain), &text::amplification(s.mic_gain), Slider::Gain, Vec::new());
    let denoise = [x, row(2.0), w, 20.0];
    match &view.denoise {
        Ok(()) => ui.button(denoise, &text::denoiser(s.denoise), Some(Action::Denoise)),
        Err(why) => {
            ui.button(denoise, text::DENOISER_MISSING, None);
            ui.tip(denoise, vec![why.clone()]);
        }
    }

    ui.icon_button([x, row(3.0), 20.0, 20.0], Icon::Mic, None, text::MIC_TEST_TIP.into());
    ui.button([x + 21.0, row(3.0), w - 21.0, 20.0], &text::activation(s.mode == Mode::Voice), Some(Action::Mode));
    let ptt = [x, row(4.0), w, 20.0];
    ui.button(ptt, &text::push_to_talk(&view.ptt_key), None);
    ui.tip(ptt, vec![text::PTT_TIP.into()]);
    let threshold = [x, row(5.0), w, 20.0];
    let meter = level_meter(threshold, view.level_db, s.threshold);
    ui.slider(threshold, threshold_value(s.threshold), &text::threshold(s.threshold), Slider::Threshold, meter);

    ui.button([x, row(6.0), half, 20.0], &text::group_names(s.group_labels), Some(Action::Labels));
    ui.button([right, row(6.0), half, 20.0], &text::protocol(&view.protocol), Some(Action::Protocol));

    let mic = [x, row(7.0), half, 20.0];
    ui.button(mic, text::SELECT_MICROPHONE, Some(Action::Open(Screen::Devices(DeviceKind::Input))));
    ui.tip(mic, vec![s.mic_device.clone().unwrap_or_else(|| text::DEFAULT_DEVICE.into())]);
    let speaker = [right, row(7.0), half, 20.0];
    ui.button(speaker, text::SELECT_SPEAKER, Some(Action::Open(Screen::Devices(DeviceKind::Output))));
    ui.tip(speaker, vec![s.speaker_device.clone().unwrap_or_else(|| text::DEFAULT_DEVICE.into())]);
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::{Screen, layout};
    use super::*;
    use crate::ui::Screens;

    fn slider_rect(widgets: &[crate::ui::Widget], s: Slider) -> [f32; 4] {
        widgets.iter().find(|w| matches!(w.action, Action::Slide(x, _) if x == s)).unwrap().rect
    }

    #[test]
    fn three_quarters_along_the_volume_slider_is_150_percent() {
        let size = [320.0, 240.0];
        let (_, widgets) = layout(&Screen::Settings, &Screens::default(), &view(), size, [0.0, 0.0], &tw);
        let r = slider_rect(&widgets, Slider::Volume);
        let mut m = Screens::default();
        m.set_widgets(widgets.clone());
        let at = [r[0] + 4.0 + 0.75 * (r[2] - 8.0), r[1] + 10.0];
        let Some(Action::Slide(Slider::Volume, v)) = m.press(at) else { panic!() };
        assert_eq!(volume_of(v), 150);
        // The threshold slider in the middle is -63 dB.
        let t = slider_rect(&widgets, Slider::Threshold);
        let Some(Action::Slide(Slider::Threshold, v)) = m.hit([t[0] + t[2] / 2.0, t[1] + 5.0]) else { panic!() };
        assert_eq!(threshold_of(v), -63);
        assert!(on_screen(&widgets, size));
    }

    #[test]
    fn one_column_at_simple_voice_chats_rows() {
        let size = [427.0, 240.0];
        let (draws, widgets) = layout(&Screen::Settings, &Screens::default(), &view(), size, [0.0, 0.0], &tw);
        let (left, top) = (((427.0 - 248.0) / 2.0_f32).floor(), ((240.0 - 219.0) / 2.0_f32).floor());
        let rel = |r: [f32; 4]| [r[0] - left, r[1] - top, r[2], r[3]];
        assert_eq!(rel(slider_rect(&widgets, Slider::Volume)), [10.0, 20.0, 228.0, 20.0]);
        assert_eq!(rel(slider_rect(&widgets, Slider::Gain)), [10.0, 41.0, 228.0, 20.0]);
        assert_eq!(rel(widget(&widgets, &Action::Denoise).unwrap()), [10.0, 62.0, 228.0, 20.0]);
        assert_eq!(rel(widget(&widgets, &Action::Mode).unwrap()), [31.0, 83.0, 207.0, 20.0]);
        assert_eq!(rel(slider_rect(&widgets, Slider::Threshold)), [10.0, 125.0, 228.0, 20.0]);
        assert_eq!(rel(widget(&widgets, &Action::Labels).unwrap()), [10.0, 146.0, 113.0, 20.0]);
        assert_eq!(rel(widget(&widgets, &Action::Protocol).unwrap()), [125.0, 146.0, 113.0, 20.0]);
        let mic = widget(&widgets, &Action::Open(Screen::Devices(DeviceKind::Input))).unwrap();
        assert_eq!(rel(mic), [10.0, 167.0, 113.0, 20.0]);
        assert!(texts(&draws).contains(&"Push To Talk: Caps Lock".to_owned()));
        // No Done button: Escape goes back, as in the Java mod in game.
        assert!(widget(&widgets, &Action::Back).is_none());
    }

    #[test]
    fn the_level_meter_fills_to_the_level() {
        let d = level_meter([0.0, 0.0, 129.0, 20.0], -30.0, -50);
        let Draw::Fill([_, _, w, _], argb) = d[0] else { panic!() };
        assert!((w / 127.0 - 0.7638).abs() < 0.001, "{w}");
        assert_eq!(argb, 0xC040_C040);
        let Draw::Fill([_, _, _, _], quiet) = level_meter([0.0, 0.0, 129.0, 20.0], -70.0, -50)[0] else { panic!() };
        assert_ne!(quiet, argb);
    }

    #[test]
    fn an_old_gain_of_200_sits_at_40_percent() {
        let s = crate::Settings::load(&|k| (k == "mic_gain").then(|| "200".to_owned()));
        assert_eq!(gain_value(s.mic_gain), 0.4);
        assert_eq!(gain_of(0.4), 200);
    }

    #[test]
    fn no_rnnoise_disables_noise_suppression_and_says_why() {
        let size = [320.0, 240.0];
        let mut v = view();
        v.denoise = Err("rnnoise.dll not found".into());
        let (draws, widgets) = layout(&Screen::Settings, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        assert!(widget(&widgets, &Action::Denoise).is_none());
        let at = text_at(&draws, "Noise Suppression: N/A").unwrap();
        let (draws, _) = layout(&Screen::Settings, &Screens::default(), &v, size, [at[0] + 2.0, at[1] + 2.0], &tw);
        assert!(texts(&draws).contains(&"rnnoise.dll not found".to_owned()));
    }
}
