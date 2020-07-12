use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{FillOptions, FillTessellator};
use lyon::tessellation::{StrokeOptions, StrokeTessellator};
use nannou::prelude::*;

use std::ops::Range;

use nannou::noise::*;

struct Model {
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    blur_render_pipeline: wgpu::RenderPipeline,
    geometry: VertexBuffers<Vertex, u16>,
    fill_count: Count,
    noise: Perlin,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    noise: [f32; 2],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let mut noise = Perlin::new();
    noise = noise.set_seed(1);

    let w_id = app.new_window().size(512, 512).view(view).build().unwrap();

    // The gpu device associated with the window's swapchain
    let window = app.window(w_id).unwrap();
    let device = window.swap_chain_device();

    // Build a Path.
    let mut builder = Path::builder();
    builder.begin(point(-0.8, -0.3));
    builder.quadratic_bezier_to(point(1.5, 2.3), point(0.2, -0.9));
    builder.end(false);
    let path = builder.build();

    let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();

    let tolerance = 0.0001;

    let mut fill_tess = FillTessellator::new();
    let fill_count = fill_tess
        .tessellate_path(
            &path,
            &FillOptions::tolerance(tolerance).with_fill_rule(tessellation::FillRule::NonZero),
            &mut BuffersBuilder::new(&mut geometry, |vertex: tessellation::FillVertex| Vertex {
                position: vertex.position().to_array(),
                noise: [0.0, 0.0],
            }),
        )
        .unwrap();

    let mut stroke_tess = StrokeTessellator::new();
    stroke_tess
        .tessellate_path(
            &path,
            &StrokeOptions::tolerance(tolerance).with_line_width(0.13),
            &mut BuffersBuilder::new(&mut geometry, |vertex: tessellation::StrokeVertex| Vertex {
                position: vertex.position().to_array(),
                noise: [0.0, 0.0],
            }),
        )
        .unwrap();

    // Create the render pipeline.
    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new().build(device);
    let bind_group = wgpu::BindGroupBuilder::new().build(device, &bind_group_layout);
    let pipeline_layout = wgpu::create_pipeline_layout(device, &[&bind_group_layout]);

    // Load shader modules.
    let vs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/shader.vert.spv"));
    let fs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/shader.frag.spv"));
    let render_pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
        .fragment_shader(&fs_mod)
        .color_format(Frame::TEXTURE_FORMAT)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float2])
        .index_format(wgpu::IndexFormat::Uint16)
        .sample_count(window.msaa_samples())
        .primitive_topology(wgpu::PrimitiveTopology::TriangleList)
        .build(device);

    let vs_mod_blur =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/blur.vert.spv"));
    let fs_mod_blur =
        wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/blur.frag.spv"));
    let blur_render_pipeline =
        wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod_blur)
            .fragment_shader(&fs_mod_blur)
            .color_format(Frame::TEXTURE_FORMAT)
            .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float2, 1 => Float2])
            .index_format(wgpu::IndexFormat::Uint16)
            .sample_count(window.msaa_samples())
            .primitive_topology(wgpu::PrimitiveTopology::TriangleList)
            .color_blend(wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            })
            .build(device);

    Model {
        bind_group,
        render_pipeline,
        blur_render_pipeline,
        geometry,
        fill_count,
        noise,
    }
}

fn update(app: &App, model: &mut Model, _: Update) {
    for vertex in model.geometry.vertices.iter_mut() {
        let cur = app.elapsed_frames() as f32 / 4000.0;
        let cur = model
            .noise
            .get([-1.0, 1.0, cur.sin() as f64, cur.cos() as f64])
            * 10.0;
        vertex.noise = [cur.sin() as f32 * 0.17, cur.cos() as f32 * 0.12];
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, model: &Model, frame: Frame) {
    let device = frame.device_queue_pair().device();

    let fill_range = 0..model.fill_count.indices;
    let stroke_range = fill_range.end..(model.geometry.indices.len() as u32);

    let vertex_buffer = device.create_buffer_with_data(
        bytemuck::cast_slice(&model.geometry.vertices),
        wgpu::BufferUsage::VERTEX,
    );

    let index_buffer = device.create_buffer_with_data(
        bytemuck::cast_slice(&model.geometry.indices),
        wgpu::BufferUsage::INDEX,
    );

    let mut staging_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("encoder 2"),
    });

    let frame_texture_view = frame.texture_view();
    let staging_texture = wgpu::TextureBuilder::new()
        .size(frame_texture_view.size())
        .build(device);
    let staging_texture_view = staging_texture.create_default_view();

    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(&staging_texture_view, |color| {
            color.clear_color(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            })
        })
        .begin(&mut staging_encoder);

    let mut encoder = frame.command_encoder();
    let mut render_pass_blur = wgpu::RenderPassBuilder::new()
        .color_attachment(frame_texture_view, |color| {
            color.clear_color(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            })
        })
        .begin(&mut encoder);

    render_pass.set_bind_group(0, &model.bind_group, &[]);

    render_pass.set_pipeline(&model.render_pipeline);
    render_pass.set_index_buffer(&index_buffer, 0, 0);
    render_pass.set_vertex_buffer(0, &vertex_buffer, 0, 0);

    // render_pass.draw_indexed(model.fill_range.clone(), 0, 0..1);
    render_pass.draw_indexed(stroke_range.clone(), 0, 0..1);

    render_pass_blur.set_bind_group(0, &model.bind_group, &[]);

    render_pass_blur.set_pipeline(&model.blur_render_pipeline);
    render_pass_blur.set_index_buffer(&index_buffer, 0, 0);
    render_pass_blur.set_vertex_buffer(0, &vertex_buffer, 0, 0);

    // render_pass.draw_indexed(model.fill_range.clone(), 0, 0..1);
    render_pass_blur.draw_indexed(stroke_range.clone(), 0, 0..1);
}
