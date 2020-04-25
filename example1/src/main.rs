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

    draw.background().color(BLUE);

    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(100.0, 100.0)
        .color(PLUM);

    draw.to_frame(app, &frame).unwrap();
}
