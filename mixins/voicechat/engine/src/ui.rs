//! The voice chat's screens (V), on the mod's own screen: a stack of them,
//! as Simple Voice Chat's (the main one, settings, groups, volumes, device
//! choice). Each frame they lay themselves out from a [`View`] of the voice
//! chat ([`crate::screens`]); this keeps only the stack, what is typed and
//! focused, the scroll and a slider being dragged.
use crate::Settings;
pub use crate::screens::{DeviceKind, Screen};

/// What a hook draws, in GUI pixels: a filled `[x, y, w, h]` in ARGB, a
/// line of text at its top left in RGB, with or without a shadow, or the part
/// `src` (`[x, y, w, h]` in the sprite's own pixels) of an atlas sprite
/// stretched into `dst`.
#[derive(Clone, Debug, PartialEq)]
pub enum Draw {
    Fill([f32; 4], u32),
    Text(String, [f32; 2], u32, bool),
    Sprite { name: String, src: [f32; 4], dst: [f32; 4] },
}

/// `draws` moved by `[dx, dy]`.
pub fn translate(draws: Vec<Draw>, d: [f32; 2]) -> Vec<Draw> {
    draws
        .into_iter()
        .map(|draw| match draw {
            Draw::Fill([x, y, w, h], c) => Draw::Fill([x + d[0], y + d[1], w, h], c),
            Draw::Text(t, [x, y], c, s) => Draw::Text(t, [x + d[0], y + d[1]], c, s),
            Draw::Sprite { name, src, dst: [x, y, w, h] } => Draw::Sprite { name, src, dst: [x + d[0], y + d[1], w, h] },
        })
        .collect()
}

/// A slider's setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slider {
    Volume,
    Gain,
    Threshold,
    Player(u128),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Mute,
    Disable,
    Mode,
    Labels,
    Denoise,
    HideIcons,
    GroupHud,
    Protocol,
    /// A slider set to a value, 0..=1 along it.
    Slide(Slider, f32),
    /// A device chosen; None is the system's default.
    Device(DeviceKind, Option<String>),
    Join(u128),
    JoinWithPassword,
    Leave,
    Focus(Field),
    Kind,
    Create,
    Scroll(i32),
    Open(Screen),
    Back,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Name,
    Password,
    JoinPassword,
    /// Adjust Volumes' search box.
    Search,
}

/// A group as the screens list it.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupRow {
    pub id: u128,
    pub name: String,
    pub has_password: bool,
    pub members: Vec<String>,
    /// The members' ids, in the same order (for their faces).
    pub member_ids: Vec<u128>,
    pub ours: bool,
    /// 0 normal, 1 open, 2 isolated.
    pub kind: i16,
}

/// A member of the player's own group, for the group screen.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberRow {
    pub id: u128,
    pub name: String,
    pub talking: bool,
    /// Voice chat turned off.
    pub disabled: bool,
    /// The player themself (no volume slider).
    pub own: bool,
    /// 0 to 400.
    pub volume: u32,
}

/// Another player with voice chat, for the volumes screen.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerRow {
    pub id: u128,
    pub name: String,
    /// 0 to 400.
    pub volume: u32,
}

/// The voice chat as the screens show it this frame.
#[derive(Clone, Debug)]
pub struct View {
    pub status: (String, u32),
    pub settings: Settings,
    pub level_db: f32,
    pub connected: bool,
    pub groups_enabled: bool,
    pub groups: Vec<GroupRow>,
    /// The player's own group's members, by name.
    pub members: Vec<MemberRow>,
    pub players: Vec<PlayerRow>,
    /// The devices of the kind the device screen lists (names).
    pub devices: Vec<String>,
    /// Whether noise suppression can run, and why not.
    pub denoise: Result<(), String>,
    /// The protocol setting, named.
    pub protocol: String,
    /// The push to talk key, named (`Caps Lock`).
    pub ptt_key: String,
}

impl Default for View {
    fn default() -> Self {
        Self {
            status: (String::new(), 0xFF_FFFF),
            settings: Settings::default(),
            level_db: -127.0,
            connected: false,
            groups_enabled: false,
            groups: Vec::new(),
            members: Vec::new(),
            players: Vec::new(),
            devices: Vec::new(),
            denoise: Ok(()),
            protocol: "Automatic".into(),
            ptt_key: "Caps Lock".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Widget {
    pub rect: [f32; 4],
    pub action: Action,
}

pub const NAME_MAX: usize = 24;
const PASSWORD_MAX: usize = 24;

pub const GREEN: u32 = 0x55_FF55;
pub const RED: u32 = 0xFF_5555;

/// The open screens, the top one shown, and what the player is doing on them.
#[derive(Default)]
pub struct Screens {
    pub stack: Vec<Screen>,
    close_requested: bool,
    pub name: String,
    pub password: String,
    pub kind: i16,
    pub focus: Option<Field>,
    /// The password group being joined, and what is typed for it.
    pub joining: Option<u128>,
    pub join_password: String,
    /// What Adjust Volumes' search box holds.
    pub search: String,
    pub message: Option<(String, u32)>,
    /// The first row a list shows.
    pub scroll: usize,
    /// The character of the key that opened the screens, typed with it.
    skip_char: Option<char>,
    widgets: Vec<Widget>,
    /// The slider being dragged and where it is.
    drag: Option<(Slider, [f32; 4])>,
}

fn inside(rect: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= rect[0] && at[0] < rect[0] + rect[2] && at[1] >= rect[1] && at[1] < rect[1] + rect[3]
}

impl Screens {
    pub fn is_open(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn top(&self) -> Option<&Screen> {
        self.stack.last()
    }

    /// Opens the main screen, opened by a key typing `opened_by`.
    pub fn open(&mut self, opened_by: Option<char>) {
        self.stack = vec![Screen::Main];
        self.skip_char = opened_by;
        self.message = None;
        self.scroll = 0;
    }

    pub fn push(&mut self, screen: Screen) {
        // The password screen is for the group being joined.
        let joining = if screen == Screen::Password { self.joining } else { None };
        self.unfocus();
        self.scroll = 0;
        self.message = None;
        self.search.clear();
        self.stack.push(screen);
        if joining.is_some() {
            self.joining = joining;
            self.focus = Some(Field::JoinPassword);
        }
    }

    /// Back to the screen before; closing when there is none.
    pub fn pop(&mut self) {
        self.unfocus();
        self.scroll = 0;
        self.drag = None;
        self.search.clear();
        self.stack.pop();
        if self.stack.is_empty() {
            self.close();
        }
    }

    fn unfocus(&mut self) {
        self.focus = None;
        self.joining = None;
        self.join_password.clear();
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.unfocus();
        self.skip_char = None;
        self.widgets.clear();
        self.drag = None;
    }

    pub fn close(&mut self) {
        self.reset();
        self.close_requested = true;
    }

    pub fn take_close(&mut self) -> bool {
        std::mem::take(&mut self.close_requested)
    }

    /// The game's screen is (`ours`) or isn't the voice chat's: forget
    /// screens the game closed.
    pub fn sync_open(&mut self, ours: bool) {
        if !ours && self.is_open() {
            self.reset();
        }
    }

    /// The widgets drawn last, for the next clicks.
    pub fn set_widgets(&mut self, widgets: Vec<Widget>) {
        self.widgets = widgets;
    }

    /// The action under the pointer, from the last frame drawn; a slider's
    /// value is where the pointer is on it.
    pub fn hit(&self, at: [f32; 2]) -> Option<Action> {
        let w = self.widgets.iter().rev().find(|w| inside(w.rect, at))?;
        Some(match w.action {
            Action::Slide(s, _) => Action::Slide(s, crate::skin::slider_value(w.rect, at[0])),
            ref other => other.clone(),
        })
    }

    /// Where the widget doing `action` was drawn last.
    pub fn hit_rect(&self, action: &Action) -> Option<[f32; 4]> {
        self.widgets.iter().find(|w| &w.action == action).map(|w| w.rect)
    }

    /// The button went down at `at`: the action there, and a slider there
    /// starts a drag.
    pub fn press(&mut self, at: [f32; 2]) -> Option<Action> {
        let action = self.hit(at)?;
        if let Action::Slide(s, _) = action {
            let rect = self.widgets.iter().rev().find(|w| inside(w.rect, at)).map(|w| w.rect)?;
            self.drag = Some((s, rect));
        }
        Some(action)
    }

    /// The pointer moved with the button down: the dragged slider's new value.
    pub fn drag(&self, at: [f32; 2]) -> Option<Action> {
        let (s, rect) = self.drag?;
        Some(Action::Slide(s, crate::skin::slider_value(rect, at[0])))
    }

    /// The button came up: the slider dragged, if one was.
    pub fn release(&mut self) -> Option<Slider> {
        self.drag.take().map(|(s, _)| s)
    }

    pub fn dragging(&self) -> Option<Slider> {
        self.drag.map(|(s, _)| s)
    }

    fn field_mut(&mut self, f: Field) -> &mut String {
        match f {
            Field::Name => &mut self.name,
            Field::Password => &mut self.password,
            Field::JoinPassword => &mut self.join_password,
            Field::Search => &mut self.search,
        }
    }

    pub fn char_typed(&mut self, c: char) {
        if let Some(skip) = self.skip_char.take()
            && skip.eq_ignore_ascii_case(&c)
        {
            return;
        }
        let Some(f) = self.focus else { return };
        let max = if matches!(f, Field::Name | Field::Search) { NAME_MAX } else { PASSWORD_MAX };
        let spaces = matches!(f, Field::Name | Field::Search);
        let field = self.field_mut(f);
        if !c.is_control() && field.chars().count() < max && (spaces || c != ' ') {
            field.push(c);
            if f == Field::Search {
                self.scroll = 0;
            }
        }
    }

    /// Editing keys. Returns the action the key asks for, if any.
    pub fn key(&mut self, key: &str, shift: bool) -> Option<Action> {
        self.skip_char = None;
        match key {
            "key.keyboard.escape" => {
                // The password screen goes back at once, as vanilla's screens
                // do with a field focused.
                if self.joining.is_some() || self.top() == Some(&Screen::Password) {
                    return Some(Action::Back);
                }
                if self.focus.is_some() {
                    self.focus = None;
                    return None;
                }
                return Some(Action::Back);
            }
            "key.keyboard.backspace" => {
                if let Some(f) = self.focus {
                    self.field_mut(f).pop();
                }
            }
            "key.keyboard.tab" => {
                self.focus = match (self.top(), self.focus, shift) {
                    (Some(Screen::Password), ..) => Some(Field::JoinPassword),
                    (Some(Screen::Volumes), ..) => Some(Field::Search),
                    (_, Some(Field::Name), false) | (_, Some(Field::Password), true) => Some(Field::Password),
                    _ => Some(Field::Name),
                };
            }
            "key.keyboard.enter" | "key.keyboard.keypad.enter" => match self.focus {
                Some(Field::JoinPassword) => return Some(Action::JoinWithPassword),
                Some(Field::Name | Field::Password) => return Some(Action::Create),
                Some(Field::Search) | None => {}
            },
            "key.keyboard.up" => self.scroll = self.scroll.saturating_sub(1),
            "key.keyboard.down" => self.scroll += 1,
            _ => {}
        }
        None
    }

    /// Whether typing goes into a field, so other keys (the menu key) type.
    pub fn typing(&self) -> bool {
        self.focus.is_some()
    }

    /// The wheel: `steps` up (positive) or down.
    pub fn scroll_by(&mut self, steps: i32) {
        self.scroll = (self.scroll as i32 - steps).max(0) as usize;
    }

    /// What a click on the screens' own parts does; the rest is the voice
    /// chat's to do (returned).
    pub fn act(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Focus(f) => self.focus = Some(f),
            Action::Kind => self.kind = (self.kind + 1) % 3,
            Action::Scroll(d) => self.scroll_by(-d),
            Action::Open(screen) => self.push(screen),
            Action::Back => self.pop(),
            Action::Close => self.close(),
            other => {
                if !matches!(other, Action::JoinWithPassword | Action::Create | Action::Slide(..)) {
                    self.focus = None;
                }
                return Some(other);
            }
        }
        None
    }

    pub fn say(&mut self, text: impl Into<String>, error: bool) {
        self.message = Some((text.into(), if error { RED } else { GREEN }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_a_new_group_and_creating_it() {
        let mut m = Screens::default();
        m.open(Some('v'));
        m.char_typed('v');
        assert_eq!(m.act(Action::Open(Screen::CreateGroup)), None);
        assert_eq!(m.top(), Some(&Screen::CreateGroup));
        assert_eq!(m.act(Action::Focus(Field::Name)), None);
        for c in "Team A".chars() {
            m.char_typed(c);
        }
        m.key("key.keyboard.tab", false);
        for c in "p w".chars() {
            m.char_typed(c);
        }
        assert_eq!((m.name.as_str(), m.password.as_str()), ("Team A", "pw"));
        m.act(Action::Kind);
        assert_eq!(m.kind, 1);
        assert_eq!(m.key("key.keyboard.enter", false), Some(Action::Create));
        // Escape leaves the field, then goes back a screen, then closes.
        assert_eq!(m.key("key.keyboard.escape", false), None);
        assert_eq!(m.key("key.keyboard.escape", false), Some(Action::Back));
    }

    #[test]
    fn the_password_screen_types_into_its_field_and_escape_leaves_it() {
        let mut m = Screens::default();
        m.open(None);
        m.act(Action::Open(Screen::Group));
        m.joining = Some(7);
        m.act(Action::Open(Screen::Password));
        assert_eq!((m.joining, m.focus), (Some(7), Some(Field::JoinPassword)));
        for c in "a b".chars() {
            m.char_typed(c);
        }
        assert_eq!(m.join_password, "ab");
        assert_eq!(m.key("key.keyboard.enter", false), Some(Action::JoinWithPassword));
        let back = m.key("key.keyboard.escape", false).unwrap();
        m.act(back);
        assert_eq!((m.top(), m.joining, m.join_password.as_str()), (Some(&Screen::Group), None, ""));
    }

    #[test]
    fn the_search_box_takes_spaces_and_goes_back_to_the_top() {
        let mut m = Screens::default();
        m.open(None);
        m.act(Action::Open(Screen::Volumes));
        m.scroll = 3;
        m.act(Action::Focus(Field::Search));
        for c in "al x".chars() {
            m.char_typed(c);
        }
        assert_eq!((m.search.as_str(), m.scroll), ("al x", 0));
        assert_eq!(m.key("key.keyboard.enter", false), None);
        m.act(Action::Back);
        assert!(m.search.is_empty());
    }

    #[test]
    fn escape_walks_back_down_the_stack_and_closes() {
        let mut m = Screens::default();
        m.open(None);
        m.act(Action::Open(Screen::Settings));
        m.act(Action::Open(Screen::Devices(DeviceKind::Input)));
        assert_eq!(m.stack.len(), 3);
        for left in [2, 1] {
            let back = m.key("key.keyboard.escape", false).unwrap();
            m.act(back);
            assert_eq!(m.stack.len(), left);
            assert!(!m.take_close());
        }
        let back = m.key("key.keyboard.escape", false).unwrap();
        m.act(back);
        assert!(!m.is_open());
        assert!(m.take_close());
    }

    #[test]
    fn a_slider_drags_until_the_button_comes_up() {
        let mut m = Screens::default();
        m.open(None);
        m.set_widgets(vec![Widget { rect: [0.0, 0.0, 108.0, 20.0], action: Action::Slide(Slider::Volume, 0.0) }]);
        assert_eq!(m.press([54.0, 10.0]), Some(Action::Slide(Slider::Volume, 0.5)));
        // Off the slider, still its value, clamped.
        assert_eq!(m.drag([500.0, 300.0]), Some(Action::Slide(Slider::Volume, 1.0)));
        assert_eq!(m.release(), Some(Slider::Volume));
        assert_eq!(m.drag([10.0, 10.0]), None);
    }
}
