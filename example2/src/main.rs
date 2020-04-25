use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run();
}

fn view(app: &App, frame: Frame) {
    let draw = app.draw();

    draw.background().color(PLUM);

    let radius = 150.0;

    let points = (0..=360).step_by(45).map(|i| {
        let radian = deg_to_rad(i as f32);
        let x = radius * radian.sin();
        let y = radius * radian.cos();
        (pt2(x, y), STEELBLUE)
    });

    draw.polygon().points_colored(points);

    draw.to_frame(app, &frame).unwrap();
}
