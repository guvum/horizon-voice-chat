//! Every line the voice chat's screens show, in one place for a later
//! translation. English, as Horizon's own interface; the wording follows
//! Simple Voice Chat's screens, so players of both find the same buttons.

pub const TITLE: &str = "Voice Chat";
pub const SETTINGS: &str = "Voice Chat Settings";
pub const JOIN_CREATE: &str = "Join/Create Group";
pub const CREATE_GROUP: &str = "Create Group";
pub const ENTER_PASSWORD: &str = "Enter Password";
pub const VOLUMES: &str = "Adjust Volumes";
pub const SELECT_MICROPHONE: &str = "Select Microphone";
pub const SELECT_SPEAKER: &str = "Select Speaker";

pub const SETTINGS_BUTTON: &str = "Settings";
pub const GROUP_BUTTON: &str = "Group";
pub const BACK: &str = "Back";
pub const CREATE: &str = "Create";
pub const JOIN_GROUP: &str = "Join Group";
pub const DEFAULT_DEVICE: &str = "Default";
pub const PLAYERS_TAB: &str = "Players";

pub const GROUP_NAME: &str = "Group Name";
pub const PASSWORD_OPTIONAL: &str = "Password (Optional)";
pub const PASSWORD: &str = "Password";
pub const SEARCH_HINT: &str = "Search...";

pub const NO_GROUPS: &str = "There are no groups";
pub const GROUPS_OFF: &str = "Groups are off on this server";
pub const NOT_CONNECTED: &str = "Voice chat isn't connected";
pub const NO_PLAYERS: &str = "No other players with voice chat";
pub const NO_MATCH: &str = "No players found";
pub const NO_DEVICES: &str = "No other devices found";
pub const NO_MEMBERS: &str = "No members";
pub const MEMBERS: &str = "Members:";

pub fn on_off(on: bool) -> &'static str {
    if on { "ON" } else { "OFF" }
}

pub fn microphone_tip(muted: bool) -> String {
    format!("Microphone: {}", if muted { "Muted" } else { "On" })
}

pub fn voice_chat_tip(disabled: bool) -> String {
    format!("Voice chat: {}", if disabled { "Off" } else { "On" })
}

pub fn hide_icons_tip(hidden: bool) -> String {
    format!("Voice chat icons: {}", if hidden { "Hidden" } else { "Shown" })
}

pub fn group_hud_tip(on: bool) -> String {
    format!("Group HUD: {}", if on { "Shown" } else { "Hidden" })
}

pub const VOLUMES_TIP: &str = "Adjust Volumes";
pub const LEAVE_TIP: &str = "Leave Group";
pub const MIC_TEST_TIP: &str = "Microphone test: not available yet";
pub const PTT_TIP: &str = "Change it in Options > Mods > Horizon Voice Chat";

pub fn volume(percent: u32) -> String {
    format!("Voice Chat Volume: {percent}%")
}

pub fn amplification(percent: u32) -> String {
    format!("Microphone Amplification: {percent}%")
}

pub fn threshold(db: i32) -> String {
    format!("Voice Activation Threshold: {db} dB")
}

pub fn activation(voice: bool) -> String {
    format!("Activation Type: {}", if voice { "Voice" } else { "Push To Talk" })
}

pub fn push_to_talk(key: &str) -> String {
    format!("Push To Talk: {key}")
}

pub fn denoiser(on: bool) -> String {
    format!("Noise Suppression: {}", on_off(on))
}

pub const DENOISER_MISSING: &str = "Noise Suppression: N/A";

pub fn group_names(on: bool) -> String {
    format!("Group Names: {}", on_off(on))
}

pub fn protocol(name: &str) -> String {
    format!("Protocol: {name}")
}

pub fn group_type(kind: i16) -> String {
    format!("Group Type: {}", KINDS[kind.rem_euclid(3) as usize])
}

pub const KINDS: [&str; 3] = ["Normal", "Open", "Isolated"];

/// What each group type means, for its button's tooltip.
pub const KIND_TIPS: [&str; 3] = [
    "Members hear each other and the players nearby",
    "Players nearby also hear the group",
    "Members hear only each other",
];

pub fn percent(n: u32) -> String {
    format!("{n}%")
}

/// A group's title: its name, and its type unless it is a normal one.
pub fn group_title(name: &str, kind: i16) -> String {
    if kind.rem_euclid(3) == 0 { name.to_owned() } else { format!("{name} ({})", KINDS[kind.rem_euclid(3) as usize]) }
}

pub fn in_group(name: &str, kind: i16) -> String {
    format!("Group: {}", group_title(name, kind))
}

pub fn and_more(n: usize) -> String {
    format!("  and {n} more")
}
