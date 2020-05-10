use nannou::prelude::*;

fn main() {
    nannou::app(model).simple_window(view).run();
}

struct Model {}

fn model(_app: &App) -> Model {
    Model {}
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    let pad = 25.0;
    let win = app.window_rect();
    let win_p = win.pad(pad);

    let square = Rect::from_w_h(100.0, 100.0).top_left_of(win_p);

    draw.rect().xy(square.xy()).wh(square.wh()).color(PLUM);

    let circle = square.below(square).shift_y(-pad);
    draw.ellipse().xy(circle.xy()).wh(circle.wh()).color(SALMON);
    draw.to_frame(app, &frame).unwrap();
}
