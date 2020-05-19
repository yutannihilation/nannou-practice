use nannou::prelude::*;
use nannou::ui::prelude::*;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}
struct Model {
    ui: Ui,
    ids: Ids,
    pitch: f32,
    yaw: f32,
    roll: f32,
    xyz: Point3<f32>,
}

widget_ids! {
    struct Ids {
        pitch,
        yaw,
        roll,
        xy,
        xz,
    }
}

fn model(app: &App) -> Model {
    // Set the loop mode to wait for events, an energy-efficient option for pure-GUI apps.
    app.set_loop_mode(LoopMode::Wait);

    // Create the UI.
    let mut ui = app.new_ui().build().unwrap();

    // Generate some ids for our widgets.
    let ids = Ids::new(ui.widget_id_generator());

    Model {
        ui,
        ids,
        pitch: 0.0,
        yaw: 0.0,
        roll: 0.0,
        xyz: pt3(0.0, 0.0, 0.0),
    }
}

fn update(app: &App, model: &mut Model, _: Update) {
    let ui = &mut model.ui.set_widgets();

    fn slider(val: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
        widget::Slider::new(val, min, max)
            .w_h(200.0, 30.0)
            .label_font_size(15)
            .rgb(0.3, 0.3, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.0)
    }

    for value in slider(model.pitch as f32, 0.0, PI)
        .top_left_with_margin(20.0)
        .label("pitch")
        .set(model.ids.pitch, ui)
    {
        model.pitch = value;
    }

    for value in slider(model.yaw, 0.0, PI)
        .down(10.0)
        .label("yaw")
        .set(model.ids.yaw, ui)
    {
        model.yaw = value;
    }

    for value in slider(model.roll, 0.0, PI)
        .down(10.0)
        .label("roll")
        .set(model.ids.roll, ui)
    {
        model.roll = value;
    }

    for (x, y) in widget::XYPad::new(model.xyz.x, -200.0, 200.0, model.xyz.y, -200.0, 200.0)
        .down(10.0)
        .w_h(200.0, 200.0)
        .label("x, y")
        .label_font_size(15)
        .rgb(0.3, 0.3, 0.3)
        .label_rgb(1.0, 1.0, 1.0)
        .border(0.0)
        .set(model.ids.xy, ui)
    {
        model.xyz = Point3::new(x, y, model.xyz.z);
    }

    for (x, z) in widget::XYPad::new(model.xyz.x, -200.0, 200.0, model.xyz.z, -200.0, 200.0)
        .down(10.0)
        .w_h(200.0, 200.0)
        .label("x, z")
        .label_font_size(15)
        .rgb(0.3, 0.3, 0.3)
        .label_rgb(1.0, 1.0, 1.0)
        .border(0.0)
        .set(model.ids.xz, ui)
    {
        model.xyz = Point3::new(x, model.xyz.y, z);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let centre = pt3(0.0, 0.0, 0.0);
    let size = vec3(100.0, 100.0, 100.0);
    let cuboid = geom::Cuboid::from_xyz_whd(centre, size);
    let points = cuboid.triangles_iter().flat_map(geom::Tri::vertices);
    draw.background().color(WHITE);
    draw.xyz(model.xyz)
        .pitch(model.pitch)
        .yaw(model.yaw)
        .roll(model.roll)
        .mesh()
        .points(points)
        .color(BLACK);

    draw.to_frame(app, &frame).unwrap();

    model.ui.draw_to_frame(app, &frame).unwrap();
}
