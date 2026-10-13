//! The Reelwright mark: the app icon (a film reel, drawn in code, see `assets/app-icon/`), decoded
//! once and kept as a texture. It is drawn where the About dialog and the Home screen carry the
//! product's mark.

const LOGO: &[u8] = include_bytes!("../../../assets/app-icon/hicolor/128x128/apps/online.jonmcgee.reelwright.png");

/// The mark's texture. (`dark` is kept from the days of a two-ink wordmark; the icon is the same on
/// both themes.) Its aspect ratio is width / height of the returned texture.
pub fn wordmark(ctx: &egui::Context, _dark: bool) -> Option<egui::TextureHandle> {
    let id = egui::Id::new("reelwright-brand-mark");
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return Some(t);
    }
    let img = image::load_from_memory_with_format(LOGO, image::ImageFormat::Png).ok()?;
    let img = img.to_rgba8();
    let color = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
    let options = egui::TextureOptions { mipmap_mode: Some(egui::TextureFilter::Linear), ..egui::TextureOptions::LINEAR };
    let tex = ctx.load_texture("reelwright-brand-mark", color, options);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    Some(tex)
}

/// Draw the mark `height` points tall with its left edge at `left_center` (vertically centred).
/// Returns the drawn rect.
pub fn paint_wordmark(ui: &egui::Ui, left_center: egui::Pos2, height: f32, dark: bool) -> Option<egui::Rect> {
    let tex = wordmark(ui.ctx(), dark)?;
    let [w, h] = tex.size();
    let width = height * w as f32 / h.max(1) as f32;
    let r = egui::Rect::from_min_size(egui::pos2(left_center.x, left_center.y - height / 2.0), egui::vec2(width, height));
    ui.painter().image(tex.id(), r, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
    Some(r)
}
