use lyon::path::builder::Build;
use nannou::prelude::*;
fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {}

fn model(_app: &App) -> Model {
    Model {}
}

fn update(_app: &App, model: &mut Model, _: Update) {}

struct MoveX {
    x: f32,
}

impl lyon::geom::traits::Transformation<f32> for MoveX {
    fn transform_point(&self, p: lyon::math::Point) -> lyon::math::Point {
        let mut p2 = p;
        p2.x += self.x;
        p2
    }

    fn transform_vector(&self, v: lyon::math::Vector) -> lyon::math::Vector {
        v
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    let win = app.window_rect();
    let builder = nannou::geom::path::Builder::new();

    draw.background().color(WHITE);
    let path_base = builder
        .move_to(pt2(0.0, 300.0))
        .quadratic_bezier_to(pt2(200.0, 0.0), pt2(-100.0, -300.0))
        // .flattened(0.9)
        .build();

    let mut path = path_base.transformed(&MoveX { x: 0.0 });
    for i in 0..=100 {
        let path_tmp = path_base
            .transformed(&MoveX {
                x: i as f32 / 100.0 * win.w(),
            })
            .reversed();
        path = path.merge(&path_tmp);
    }
    draw.path().fill().color(BLACK).events(path.iter());

    draw.to_frame(app, &frame).unwrap();
}
