use nannou::prelude::*;

use rapier2d::data::arena::Index;
use rapier2d::dynamics::{
    BallJoint, FixedJoint, IntegrationParameters, JointSet, RigidBodyBuilder, RigidBodySet,
};
use rapier2d::geometry::{BroadPhase, ColliderBuilder, ColliderSet, NarrowPhase};
use rapier2d::na::{Isometry2, Point2, Vector2};
use rapier2d::pipeline::PhysicsPipeline;

use lyon::math::point;
use lyon::path::Event::*;
use lyon::path::Path;

use std::f32::consts::PI;

const TOLERANCE: f32 = 0.01;
const HEIGHT: f32 = 10.0;

struct Model {
    pipeline: PhysicsPipeline,
    gravity: Vector2<f32>,
    integration_parameters: IntegrationParameters,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    joints: JointSet,

    point_indices: Vec<Vec<Index>>,

    record: bool,
    recording_frame: u32,
}

struct Builder {
    cur_path_id: u32,
    cur_glyph_id: u32,
    offset_x: f32,
    offset_y: f32,
    tolerance: f32,
    builder: lyon::path::BuilderWithAttributes,
}

impl Builder {
    fn new(tolerance: f32) -> Self {
        let builder = Path::builder_with_attributes(2);
        Self {
            cur_path_id: 0,
            cur_glyph_id: 0,
            offset_x: 0.0,
            offset_y: 0.0,
            tolerance,
            builder,
        }
    }

    // rusttype returns the positions to the origin, so we need to
    // move to each offsets by ourselves
    fn point(&self, x: f32, y: f32) -> lyon::math::Point {
        point(x + self.offset_x, self.offset_y - y)
    }

    fn next_glyph(&mut self, glyph_id: u32, bbox: &rusttype::Rect<i32>) {
        self.cur_glyph_id = glyph_id;
        self.offset_x = bbox.min.x as _;
        self.offset_y = bbox.max.y as _;
    }

    fn to_path(self) -> Vec<Vec<Point2<f32>>> {
        let path = self.builder.build();

        let mut result = vec![];
        let mut points: Vec<Point2<f32>> = vec![];

        for p in path.iter_with_attributes() {
            println!("{:?}", p);
            match p {
                Begin { .. } => {}
                Line { from, to } => points.push(Point2::new(to.0.x, to.0.y)),
                Quadratic { from, ctrl, to } => {
                    let seg = lyon::geom::QuadraticBezierSegment {
                        from: from.0,
                        ctrl,
                        to: to.0,
                    };
                    // skip the first point as it's already added
                    for p in seg.flattened(self.tolerance).skip(1) {
                        points.push(Point2::new(p.x, p.y))
                    }
                }
                Cubic {
                    from,
                    ctrl1,
                    ctrl2,
                    to,
                } => {
                    let seg = lyon::geom::CubicBezierSegment {
                        from: from.0,
                        ctrl1,
                        ctrl2,
                        to: to.0,
                    };
                    // skip the first point as it's already added
                    for p in seg.flattened(self.tolerance).skip(1) {
                        points.push(Point2::new(p.x, p.y))
                    }
                }
                End { last, first, close } => {
                    // points.push(Point2::new(last.0.x, last.0.y));
                    result.push(points.clone());
                    points.clear();
                }
            };
        }
        result
    }
}

impl<'a> rusttype::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.move_to(
            self.point(x, y),
            &[self.cur_path_id as _, self.cur_glyph_id as _],
        );
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(
            self.point(x, y),
            &[self.cur_path_id as _, self.cur_glyph_id as _],
        );
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.builder.quadratic_bezier_to(
            self.point(x1, y1),
            self.point(x, y),
            &[self.cur_path_id as _, self.cur_glyph_id as _],
        );
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.builder.cubic_bezier_to(
            self.point(x1, y1),
            self.point(x2, y2),
            self.point(x, y),
            &[self.cur_path_id as _, self.cur_glyph_id as _],
        );
    }

    fn close(&mut self) {
        self.cur_path_id += 1;
        self.builder.close();
    }
}

fn main() {
    nannou::app(model)
        .event(event)
        .simple_window(view)
        .update(update)
        .run();
}

fn model(_app: &App) -> Model {
    let font = rusttype::Font::try_from_bytes(include_bytes!(
        // "/usr/share/fonts/TTF/iosevka-heavyitalic.ttf"
        // "../fonts/UniHentaiKana-Regular.otf"
        // "/usr/share/fonts/gsfonts/C059-Roman.otf"
        "../fonts/ipam.ttf"
    ))
    .unwrap();

    let scale = rusttype::Scale::uniform(HEIGHT);
    let v_metrics = font.v_metrics(scale);
    let offset = rusttype::point(0.0, v_metrics.ascent);

    let mut glyph = font.layout("落", scale, offset);
    let mut builder = Builder::new(TOLERANCE);

    for (glyph_id, g) in glyph.enumerate() {
        if let Some(bbox) = g.pixel_bounding_box() {
            // bbox_y.push(bbox.max.y);
            builder.next_glyph(glyph_id as _, &bbox);
        } else {
            continue;
        }

        if !g.build_outline(&mut builder) {
            println!("empty");
        }
    }

    let font_points = builder.to_path();

    let pipeline = PhysicsPipeline::new();
    let gravity = Vector2::new(0.0, -9.81);
    let integration_parameters = IntegrationParameters::default();
    let broad_phase = BroadPhase::new();
    let narrow_phase = NarrowPhase::new();
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let mut joints = JointSet::new();
    // let event_handler = ();

    // Add ground
    let ground = RigidBodyBuilder::new_static()
        .position(Isometry2::new(Vector2::new(0.0, -5.0), PI))
        .build();
    let idx_ground = bodies.insert(ground);
    let coll_ground = ColliderBuilder::capsule_x(100.0, 1.0)
        .friction(0.8)
        .density(100.0)
        .build();
    // ColliderBuilder::segment(Point2::new(-1.0, 0.0), Point2::new(1.0, 0.0)).build();
    colliders.insert(coll_ground, idx_ground, &mut bodies);

    let mut point_indices = vec![];

    for points in font_points {
        let mut point_indices_inner = vec![];
        // Add points
        for pos in points.iter() {
            let vec = pos - Point2::origin();

            let rot = (pos - Point2::new(3.2, 2.2)) / 4.0;

            let p = RigidBodyBuilder::new_dynamic()
                // Note: the unit of position is meter, as I set the gravity to -9.81.
                .position(Isometry2::new(vec, PI))
                .linvel(-0.5 - rot[1], 3.0 + rot[0])
                .build();

            let idx = bodies.insert(p);
            let coll = ColliderBuilder::ball(0.1).build();
            colliders.insert(coll, idx, &mut bodies);

            point_indices_inner.push(idx);
        }

        // Add joints
        // let p_len = point_indices_inner.len();
        // for i in 0..p_len {
        //     let idx = point_indices_inner[i];
        //     let p = bodies.get(idx).unwrap().position.to_homogeneous();

        //     let idx_next = point_indices_inner[(i + 1) % p_len];
        //     let p_next = bodies.get(idx_next).unwrap().position.to_homogeneous();

        //     // let joint_params = BallJoint::new(
        //     //     Point2::new(-p[(0, 2)], p[(1, 2)] / 10.0),
        //     //     Point2::new(p_next[(0, 2)] / 10.0, p_next[(1, 2)] / 10.0),
        //     // );
        //     let joint_params = BallJoint::new(Point2::new(3.0, 0.0), Point2::new(0.0, 3.0));
        //     // let joint_params = FixedJoint::new(p.position, p_next.position);
        //     joints.insert(&mut bodies, idx, idx_next, joint_params);
        // }

        point_indices.push(point_indices_inner);
    }

    Model {
        pipeline,
        gravity,
        integration_parameters,
        broad_phase,
        narrow_phase,
        bodies,
        colliders,
        joints,

        point_indices,

        record: true,
        recording_frame: 0,
    }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if model.record {
        if model.recording_frame >= 999 {
            println!("Finish recording");

            model.record = false;
            model.recording_frame = 0;
        } else {
            model.recording_frame += 1;
        }
    }

    model.pipeline.step(
        &model.gravity,
        &model.integration_parameters,
        &mut model.broad_phase,
        &mut model.narrow_phase,
        &mut model.bodies,
        &mut model.colliders,
        &mut model.joints,
        &(),
    );
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(nannou::color::rgb_u32(0xDEC2CB));

    for point_indices in &model.point_indices {
        let p_len = point_indices.len();
        for i in 0..p_len {
            let idx = point_indices[i];

            // add point
            let p = model.bodies.get(idx).unwrap();
            let p_homo = p.position.to_homogeneous() * 40.0;
            draw.ellipse()
                .x_y(p_homo[(0, 2)], p_homo[(1, 2)])
                .radius(4.5)
                .color(nannou::color::rgb_u32(0x91163D));

            // if there's only one point, there's no lines
            if p_len == 1 {
                break;
            }

            let idx_next = point_indices[(i + 1) % p_len];
            let p_next = model.bodies.get(idx_next).unwrap();
            let p_next_homo = p_next.position.to_homogeneous() * 40.0;
            draw.line()
                .start(pt2(p_homo[(0, 2)], p_homo[(1, 2)]))
                .end(pt2(p_next_homo[(0, 2)], p_next_homo[(1, 2)]))
                .weight(3.2)
                .color(nannou::color::rgb_u32(0x91163D));
        }
    }

    draw.to_frame(app, &frame).unwrap();

    // Capture the frame!
    if model.record {
        let file_path = captured_frame_path(app, model.recording_frame);
        app.main_window().capture_frame(file_path);
    }
}

fn captured_frame_path(app: &App, frame: u32) -> std::path::PathBuf {
    // Create a path that we want to save this frame to.
    app.project_path()
        .expect("failed to locate `project_path`")
        .join("img")
        .join(format!("{:03}", frame))
        .with_extension("png")
}
