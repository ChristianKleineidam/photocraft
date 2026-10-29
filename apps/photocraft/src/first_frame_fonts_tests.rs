//! Exercise the real logic/UI callbacks to catch an accidentally blank first editor frame.

use eframe::{App as _, egui};
use photocraft_ui_egui::{PhotocraftApp, Services, i18n};

/// Run one complete UI pass without a native window or graphics device.
fn draw(app: &mut PhotocraftApp, ctx: &egui::Context) -> egui::FullOutput {
    let mut frame = eframe::Frame::_new_kittest();
    let mut input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1440.0, 900.0))), ..Default::default() };
    app.raw_input_hook(ctx, &mut input);
    ctx.run_ui(input, |ui| {
        app.logic(ui.ctx(), &mut frame);
        app.ui(ui, &mut frame);
    })
}

/// App-creator preparation must make the first presentation contain editor text, with its
/// named weights already active, rather than spend that presentation installing fonts.
#[test]
fn prepared_context_draws_editor_text_on_the_first_pass() {
    let _language = i18n::language_scope(i18n::Lang::EN);
    let ctx = egui::Context::default();
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Services::default());
    app.prepare_first_frame(&ctx);
    let output = draw(&mut app, &ctx);
    assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(_))), "first presentation must draw editor text");
    ctx.fonts(|fonts| {
        for name in ["medium", "semibold"] {
            assert!(fonts.families().contains(&egui::FontFamily::Name(name.into())), "missing first-pass family: {name}");
        }
    });
}

/// Hosts that cannot prepare before begin-pass retain the safe blank-then-editor fallback;
/// drawing named font families in that first unprepared pass would panic inside egui.
#[test]
fn unprepared_context_keeps_the_deferred_font_setup() {
    let _language = i18n::language_scope(i18n::Lang::EN);
    let ctx = egui::Context::default();
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Services::default());
    let first = draw(&mut app, &ctx);
    assert!(!first.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(_))));
    let second = draw(&mut app, &ctx);
    assert!(second.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(_))));
}

/// Saved font size must apply to the first visible editor frame and stay stable afterward,
/// rather than briefly drawing Small or accumulating another scaling factor every pass.
#[test]
fn saved_font_size_is_active_on_the_first_pass() {
    let _language = i18n::language_scope(i18n::Lang::EN);
    let ctx = egui::Context::default();
    let services = Services { load_prefs: Some(Box::new(|| Some(r#"{"interface":{"uiFontSize":"large","language":"en"}}"#.into()))), ..Default::default() };
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
    app.prepare_first_frame(&ctx);
    for pass in 1..=3 {
        let output = draw(&mut app, &ctx);
        assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(_))), "pass {pass}");
        ctx.fonts(|fonts| {
            let scale = fonts.definitions().font_data.get("Inter").expect("Inter installed").tweak.scale;
            assert!((scale - 16.0 / 12.0).abs() < 0.0001, "pass {pass}: scale {scale}");
        });
    }
}

/// Replacing a host context must register resources for that new context before its first
/// pass, even when the app has already rendered on another context.
#[test]
fn replacement_context_also_draws_its_first_pass() {
    let _language = i18n::language_scope(i18n::Lang::EN);
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Services::default());
    for _ in 0..2 {
        let ctx = egui::Context::default();
        app.prepare_first_frame(&ctx);
        let output = draw(&mut app, &ctx);
        assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(_))));
    }
}
