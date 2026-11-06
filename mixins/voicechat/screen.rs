// Draws the voice chat's screen on top (main, settings, groups, group
// creation and password, volumes, devices) with vanilla's button, slider and text field sprites and the
// mod's icons; a sprite the atlas lacks gets a plain stand-in.
fn voicechat_screen(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use voicechat_engine as vc;
    if game.native_mod_screen() != Some(vc::ID) {
        return;
    }
    vc::menu_ptt_key(&game.native_mod_key(vc::ID, vc::KEY_PTT, vc::DEFAULT_PTT));
    let measure = |t: &str| game.native_text_width(t);
    let draws = vc::menu_draw([batch.width, batch.height], screen.mouse, &measure);
    vc::render(&draws, &mut |d| match d {
        vc::ui::Draw::Fill(rect, argb) => {
            game.native_fill(batch, *rect, *argb);
            true
        }
        vc::ui::Draw::Text(text, at, rgb, shadow) => {
            game.native_text(batch, text, *at, *rgb, *shadow);
            true
        }
        vc::ui::Draw::Sprite { name, src, dst } => game.native_sprite(batch, name, *src, *dst),
    });
}
