use nannou::prelude::*;

use rapier2d::data::arena::Index;
use rapier2d::dynamics::{
    BallJoint, FixedJoint, IntegrationParameters, JointSet, RigidBodyBuilder, RigidBodySet,
};
use rapier2d::geometry::{BroadPhase, ColliderBuilder, ColliderSet, NarrowPhase};
use rapier2d::na::{Isometry2, Point2, Vector2};
use rapier2d::pipeline::PhysicsPipeline;

use std::f32::consts::PI;

struct Model {
    pipeline: PhysicsPipeline,
    gravity: Vector2<f32>,
    integration_parameters: IntegrationParameters,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    joints: JointSet,

    point_indices: Vec<Index>,

    record: bool,
    recording_frame: u32,
}

fn main() {
    nannou::app(model)
        .event(event)
        .simple_window(view)
        .update(update)
        .run();
}

fn model(_app: &App) -> Model {
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
        .position(Isometry2::new(Vector2::new(0.0, -1.0), PI))
        .build();
    let idx_ground = bodies.insert(ground);
    let coll_ground = ColliderBuilder::capsule_x(10.0, 0.0).friction(0.8).build();
    // ColliderBuilder::segment(Point2::new(-1.0, 0.0), Point2::new(1.0, 0.0)).build();
    colliders.insert(coll_ground, idx_ground, &mut bodies);

    // Add points
    let points = vec![
        Vector2::new(1.0, 2.0),
        Vector2::new(-1.0, 3.0),
        Vector2::new(-1.1, 5.0),
        Vector2::new(1.1, 3.0),
    ];
    let point_indices = points
        .iter()
        .map(|&pos| {
            let p = RigidBodyBuilder::new_dynamic()
                // Note: the unit of position is meter, as I set the gravity to -9.81.
                .position(Isometry2::new(pos, PI))
                .linvel(0.0, 1.0)
                .build();

            let idx = bodies.insert(p);
            let coll = ColliderBuilder::ball(0.3).build();
            colliders.insert(coll, idx, &mut bodies);
            idx
        })
        .collect::<Vec<_>>();

    // Add joints
    let p_len = point_indices.len();
    for i in 0..p_len {
        let idx = point_indices[i];
        let p = bodies.get(idx).unwrap().position.to_homogeneous();

        let idx_next = point_indices[(i + 1) % p_len];
        let p_next = bodies.get(idx_next).unwrap().position.to_homogeneous();

        let joint_params = BallJoint::new(
            Point2::new(p[(0, 2)], p[(1, 2)]),
            Point2::new(p_next[(0, 2)], p_next[(1, 2)]),
        );
        // let joint_params = FixedJoint::new(p.position, p_next.position);
        joints.insert(&mut bodies, idx, idx_next, joint_params);
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

fn update(app: &App, model: &mut Model, update: Update) {
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

    let p_len = model.point_indices.len();
    for i in 0..p_len {
        let idx = model.point_indices[i];

        // add point
        let p = model.bodies.get(idx).unwrap();
        let p_homo = p.position.to_homogeneous() * 100.0;
        draw.ellipse()
            .x_y(p_homo[(0, 2)], p_homo[(1, 2)])
            .radius(30.0)
            .color(nannou::color::rgb_u32(0x91163D));

        // if there's only one point, there's no lines
        if p_len == 1 {
            break;
        }

        let idx_next = model.point_indices[(i + 1) % p_len];
        let p_next = model.bodies.get(idx_next).unwrap();
        let p_next_homo = p_next.position.to_homogeneous() * 100.0;
        draw.line()
            .start(pt2(p_homo[(0, 2)], p_homo[(1, 2)]))
            .end(pt2(p_next_homo[(0, 2)], p_next_homo[(1, 2)]))
            .weight(4.0)
            .color(nannou::color::rgb_u32(0x91163D));
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
