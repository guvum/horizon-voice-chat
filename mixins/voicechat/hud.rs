// The HUD: the voice chat's icon near the bottom left corner where Simple
// Voice Chat puts it (microphone, muted, off, disconnected, whispering) with
// what doesn't work above it, the group's members in the top left corner,
// each player's group above their head and the voice chat icon right of
// their name tag (talking, whispering, off...), seen from the camera the
// world was drawn from.
fn voicechat_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use horizon_client::extras::Corner;
    use voicechat_engine as vc;
    if !game.native_on(vc::ID) {
        return;
    }
    let game = &*game;
    let size = [batch.width, batch.height];
    let mut draws = Vec::new();
    if let Some((pos, yaw, pitch, fov)) = crate::game::remote::voicechat_world::camera(game) {
        let camera = vc::labels::Camera { pos, yaw, pitch, fov };
        for label in vc::group_labels(&camera, size) {
            let w = game.native_text_width(&label.text) + 4.0;
            let [x, y] = [label.at[0] - w / 2.0, label.at[1] - 6.0];
            draws.push(vc::ui::Draw::Fill([x, y, w, 12.0], 0x6000_0000));
            draws.push(vc::ui::Draw::Text(label.text, [x + 2.0, y + 2.0], label.rgb, true));
        }
        draws.extend(vc::talk_icons(&camera, size, &|t: &str| game.native_text_width(t)));
    }
    let measure = |t: &str| game.native_text_width(t);
    let hud = vc::hud(size, &measure);
    draws.extend(hud.status);
    draws.extend(hud.problems);
    if !hud.group.is_empty() && game.native_corner_shown(Corner::TopLeft) {
        let (x, y) = corners.place(Corner::TopLeft, hud.group_size[0], hud.group_size[1]);
        draws.extend(vc::ui::translate(hud.group, [x, y]));
    }
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
