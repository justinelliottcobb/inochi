use nannou::prelude::*;

fn main() {
    nannou::app(model)
        .update(update)
        .view(view)
        .raw_window_event(raw_window_event)
        .run();
}

fn model(app: &App) -> inochi::App {
    let window_id = app
        .new_window()
        .title("Inochi - Particle Life System")
        .size(1200, 800)
        .decorations(true)
        .resizable(true)
        .build()
        .unwrap();

    inochi::App::new(app, window_id)
}

fn update(app: &App, model: &mut inochi::App, update: Update) {
    model.update(app, &update);
}

fn view(app: &App, model: &mut inochi::App, frame: Frame) {
    model.view(app, &frame);
}

fn raw_window_event(app: &App, model: &mut inochi::App, event: &nannou::winit::event::WindowEvent) {
    model.raw_window_event(app, event);
}
