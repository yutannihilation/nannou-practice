use itertools::iproduct;
use nannou::prelude::*;
extern crate nalgebra as na;

fn main() {
    nannou::app(model).simple_window(view).run();
}

struct Model {}

fn model(app: &App) -> Model {
    Model {}
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let cube = iproduct!(0..=1, 0..=1, 0..=1)
        .map(|(x, y, z)| na::Point3::new((x * 100) as f32, (y * 100) as f32, (z * 100) as f32));

    let eye = na::Point3::new(0.0, 0.0, 1.0);
    let target = na::Point3::new(0.01, 0.0, 0.0);
    let view = na::Isometry3::look_at_rh(&eye, &target, &na::Vector3::y());

    let model = na::Isometry3::new(na::Vector3::x(), na::zero());
    let projection = na::Perspective3::new(16.0 / 9.0, 3.14 / 2.0, 1.0, 1000.0);
    let model_view_projection = projection.as_matrix() * (view * model).to_homogeneous();

    for p in cube {
        let v: [f32; 2] = (model_view_projection.transform_point(&p) - na::Point3::origin())
            .xy()
            .into();
        draw.ellipse().xy(v.into()).w_h(1.0, 1.0).color(WHITE);
    }

    draw.to_frame(app, &frame).unwrap();
}
