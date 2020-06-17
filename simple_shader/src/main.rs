use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{FillOptions, FillTessellator};
use lyon::tessellation::{StrokeOptions, StrokeTessellator};
use nannou::prelude::*;

use std::ops::Range;

struct Model {
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    fill_range: Range<u32>,
    stroke_range: Range<u32>,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    let w_id = app.new_window().size(512, 512).view(view).build().unwrap();

    // The gpu device associated with the window's swapchain
    let window = app.window(w_id).unwrap();
    let device = window.swap_chain_device();
    let format = Frame::TEXTURE_FORMAT;
    let sample_count = window.msaa_samples();

    let tolerance = 0.01;

    let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();

    let mut fill_tess = FillTessellator::new();
    let mut stroke_tess = StrokeTessellator::new();

    // Build a Path.
    let mut builder = Path::builder();
    builder.begin(point(-0.1, -0.1));
    builder.line_to(point(0.1, -0.1));
    // builder.quadratic_bezier_to(point(0.2, 0.0), point(0.2, 0.1));
    // builder.cubic_bezier_to(point(0.1, 0.1), point(0.0, 0.1), point(0.0, 0.0));
    builder.end(false);
    let path = builder.build();

    let fill_count = fill_tess
        .tessellate_path(
            &path,
            &FillOptions::tolerance(tolerance).with_fill_rule(tessellation::FillRule::NonZero),
            &mut BuffersBuilder::new(&mut geometry, |vertex: tessellation::FillVertex| Vertex {
                position: vertex.position().to_array(),
            }),
        )
        .unwrap();

    stroke_tess
        .tessellate_path(
            &path,
            &StrokeOptions::tolerance(tolerance),
            &mut BuffersBuilder::new(&mut geometry, |vertex: tessellation::StrokeVertex| Vertex {
                position: vertex.position().to_array(),
            }),
        )
        .unwrap();

    let fill_range = 0..fill_count.indices;
    let stroke_range = fill_range.end..(geometry.indices.len() as u32);

    let vertex_buffer = device.create_buffer_with_data(
        bytemuck::cast_slice(&geometry.vertices),
        wgpu::BufferUsage::VERTEX,
    );

    let index_buffer = device.create_buffer_with_data(
        bytemuck::cast_slice(&geometry.indices),
        wgpu::BufferUsage::INDEX,
    );

    // Load shader modules.
    let vs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/vert.spv"));
    let fs_mod = wgpu::shader_from_spirv_bytes(device, include_bytes!("shaders/frag.spv"));

    // Create the render pipeline.
    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new().build(device);
    let bind_group = wgpu::BindGroupBuilder::new().build(device, &bind_group_layout);
    let pipeline_layout = wgpu::create_pipeline_layout(device, &[&bind_group_layout]);
    let render_pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
        .fragment_shader(&fs_mod)
        .color_format(format)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float2])
        .index_format(wgpu::IndexFormat::Uint16)
        .sample_count(sample_count)
        .primitive_topology(wgpu::PrimitiveTopology::LineList)
        .build(device);

    Model {
        bind_group,
        render_pipeline,
        vertex_buffer,
        index_buffer,
        fill_range,
        stroke_range,
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, model: &Model, frame: Frame) {
    let mut encoder = frame.command_encoder();

    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color)
        .begin(&mut encoder);

    render_pass.set_bind_group(0, &model.bind_group, &[]);
    render_pass.set_pipeline(&model.render_pipeline);
    render_pass.set_index_buffer(&model.index_buffer, 0, 0);
    render_pass.set_vertex_buffer(0, &model.vertex_buffer, 0, 0);

    render_pass.draw_indexed(model.fill_range.clone(), 0, 0..1);
    render_pass.draw_indexed(model.stroke_range.clone(), 0, 0..1);
}
