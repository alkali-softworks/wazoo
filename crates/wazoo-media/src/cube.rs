/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * 3D Video Cube Pipeline
 *
 * Renders 3D bouncing cubes with live video textures mapped to all six faces,
 * directional lighting, specular highlights, and glowing neon edges.
 */

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

use bytemuck::{Pod, Zeroable};
use iced::advanced::graphics::Viewport;
use iced::widget::shader::{Pipeline, Primitive, Program, Shader};
use iced::{Length, Rectangle, mouse};
use iced_wgpu::wgpu;

use crate::pipeline::FrameData;

/// GPU uniform buffer layout for the 3D cube vertex and fragment shaders.
/// Matches `struct CubeUniforms` in WGSL, strictly aligned to 16-byte boundaries (80 bytes total).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CubeUniforms {
    pub screen_size: [f32; 2],
    pub cube_pos: [f32; 2],
    pub cube_size: f32,
    pub time: f32,
    pub _pad0: [f32; 2],
    pub rotation: [f32; 3],
    pub has_texture: f32,
    pub edge_color: [f32; 4],
    pub opacity: f32,
    pub crt_enabled: f32,
    pub sheen_enabled: f32,
    pub _pad1: f32,
}

struct CubeGpuEntry {
    texture: wgpu::Texture,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
    alive: Arc<AtomicBool>,
    last_frame_seq: u64,
}

/// Manages WGPU render pipeline state, texture sampling, and GPU buffer lifecycle for 3D cubes.
pub struct CubePipeline {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    texture_format: wgpu::TextureFormat,
    entries: BTreeMap<u64, CubeGpuEntry>,
}

const CUBE_SHADER_SRC: &str = r#"
struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
}

struct CubeUniforms {
    screen_size: vec2<f32>,
    cube_pos: vec2<f32>,
    cube_size: f32,
    time: f32,
    _pad0: vec2<f32>,
    rotation: vec3<f32>,
    has_texture: f32,
    edge_color: vec4<f32>,
    opacity: f32,
    crt_enabled: f32,
    sheen_enabled: f32,
    _pad1: f32,
}

@group(0) @binding(0)
var tex: texture_2d<f32>;

@group(0) @binding(1)
var s: sampler;

@group(0) @binding(2)
var<uniform> uniforms: CubeUniforms;

fn rotate_x(p: vec3<f32>, a: f32) -> vec3<f32> {
    let c = cos(a);
    let s_val = sin(a);
    return vec3<f32>(p.x, p.y * c - p.z * s_val, p.y * s_val + p.z * c);
}

fn rotate_y(p: vec3<f32>, a: f32) -> vec3<f32> {
    let c = cos(a);
    let s_val = sin(a);
    return vec3<f32>(p.x * c + p.z * s_val, p.y, -p.x * s_val + p.z * c);
}

fn rotate_z(p: vec3<f32>, a: f32) -> vec3<f32> {
    let c = cos(a);
    let s_val = sin(a);
    return vec3<f32>(p.x * c - p.y * s_val, p.x * s_val + p.y * c, p.z);
}

fn rotate_3d(p: vec3<f32>, angles: vec3<f32>) -> vec3<f32> {
    var v = rotate_x(p, angles.x);
    v = rotate_y(v, angles.y);
    v = rotate_z(v, angles.z);
    return v;
}

const CAM_DIST: f32 = 3.8;
const FOV_SCALE: f32 = 3.8;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    // 6 faces * 6 vertices = 36 vertices
    // Model vertices [-1.0, 1.0]
    var positions = array<vec3<f32>, 36>(
        // Front (+Z)
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>(-1.0, -1.0,  1.0),

        // Back (-Z)
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>( 1.0, -1.0, -1.0),

        // Top (+Y)
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>(-1.0,  1.0,  1.0),

        // Bottom (-Y)
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>(-1.0, -1.0, -1.0),

        // Right (+X)
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>( 1.0, -1.0,  1.0),

        // Left (-X)
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
    );

    var uvs = array<vec2<f32>, 36>(
        // Front
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
        // Back
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
        // Top
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
        // Bottom
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
        // Right
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
        // Left
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );

    var normals = array<vec3<f32>, 6>(
        vec3<f32>( 0.0,  0.0,  1.0), // Front
        vec3<f32>( 0.0,  0.0, -1.0), // Back
        vec3<f32>( 0.0,  1.0,  0.0), // Top
        vec3<f32>( 0.0, -1.0,  0.0), // Bottom
        vec3<f32>( 1.0,  0.0,  0.0), // Right
        vec3<f32>(-1.0,  0.0,  0.0), // Left
    );

    let pos = positions[idx];
    let uv = uvs[idx];
    let norm = normals[idx / 6u];

    let rotated_pos = rotate_3d(pos, uniforms.rotation);
    let rotated_norm = rotate_3d(norm, uniforms.rotation);

    // True perspective projection from camera at (0, 0, CAM_DIST) looking towards origin
    let pers_scale = FOV_SCALE / (CAM_DIST - rotated_pos.z);
    let proj_2d = rotated_pos.xy * pers_scale;

    // Projected position in screen pixels (Y inverted: screen Y increases downwards)
    let pixel_pos = vec2<f32>(
        uniforms.cube_pos.x + proj_2d.x * uniforms.cube_size,
        uniforms.cube_pos.y - proj_2d.y * uniforms.cube_size
    );

    // Convert pixel coordinates (0..W, 0..H with 0 top) to NDC (-1..1 with 1 top)
    let ndc_x = (pixel_pos.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_pos.y / uniforms.screen_size.y) * 2.0;
    let depth = clamp(((rotated_pos.z + 2.0) / 4.0), 0.0, 1.0);

    var out: VertexOutput;
    out.clip_pos = vec4<f32>(ndc_x, ndc_y, depth, 1.0);
    out.uv = uv;
    out.normal = rotated_norm;
    out.world_pos = rotated_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Backface culling in perspective view:
    // Discard any face pointing away from camera at (0, 0, CAM_DIST)
    let view_dir = normalize(vec3<f32>(0.0, 0.0, CAM_DIST) - in.world_pos);
    if (dot(in.normal, view_dir) <= 0.0) {
        discard;
    }

    var col: vec4<f32>;
    if (uniforms.has_texture > 0.5) {
        col = textureSampleLevel(tex, s, in.uv, 0.0);
    } else {
        // Procedural retro cyberpunk test pattern:
        // Animated grid lines + gradient + color bars
        let grid = step(vec2<f32>(0.92, 0.92), fract(in.uv * 8.0));
        let grid_line = max(grid.x, grid.y);
        let bar_idx = floor(in.uv.x * 7.0);
        var bar_color = vec3<f32>(0.2, 0.2, 0.2);
        if (bar_idx == 0.0) { bar_color = vec3<f32>(0.9, 0.9, 0.9); }
        else if (bar_idx == 1.0) { bar_color = vec3<f32>(0.9, 0.8, 0.2); }
        else if (bar_idx == 2.0) { bar_color = vec3<f32>(0.2, 0.8, 0.8); }
        else if (bar_idx == 3.0) { bar_color = vec3<f32>(0.2, 0.8, 0.2); }
        else if (bar_idx == 4.0) { bar_color = vec3<f32>(0.8, 0.2, 0.8); }
        else if (bar_idx == 5.0) { bar_color = vec3<f32>(0.8, 0.2, 0.2); }
        else { bar_color = vec3<f32>(0.2, 0.2, 0.8); }

        let sweep = sin(uniforms.time * 3.0 + in.uv.y * 10.0) * 0.15;
        let base = bar_color + sweep + grid_line * 0.3;
        col = vec4<f32>(base, 1.0);
    }

    var final_rgb = col.rgb;

    // 3D Directional Lighting + Specular Sheen (toggleable)
    if (uniforms.sheen_enabled > 0.5) {
        let light_dir = normalize(vec3<f32>(0.4, 0.7, 0.9));
        let diff = max(dot(in.normal, light_dir), 0.0);
        let lighting = diff * 0.35 + 0.75;
        final_rgb = final_rgb * lighting;

        // Specular highlight / sheen
        let half_vec = normalize(light_dir + view_dir);
        let spec = pow(max(dot(in.normal, half_vec), 0.0), 16.0) * 0.25;
        final_rgb += vec3<f32>(spec);
    }

    // CRT scanline effect on the cube if enabled
    if (uniforms.crt_enabled > 0.5) {
        let scanline = sin(in.uv.y * 240.0 + uniforms.time * 6.0) * 0.12;
        final_rgb -= vec3<f32>(scanline);
    }

    // Active player focus outline flash (green outline matching normal players when selected)
    if (uniforms.edge_color.a > 0.0) {
        let edge_dist = min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y));
        let border_width = 0.038;
        if (edge_dist < border_width) {
            let edge_blend = smoothstep(0.0, 0.006, border_width - edge_dist) * uniforms.edge_color.a;
            let focus_glow = uniforms.edge_color.rgb * 1.25;
            final_rgb = mix(final_rgb, focus_glow, edge_blend);
        }
    }

    return vec4<f32>(clamp(final_rgb, vec3<f32>(0.0), vec3<f32>(1.0)), col.a * uniforms.opacity);
}
"#;

impl CubePipeline {
    fn create_entry(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        alive: Arc<AtomicBool>,
    ) -> CubeGpuEntry {
        let w = width.max(1);
        let h = height.max(1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("wazoo cube video texture"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Initialize with default pixels
        let init_pixels = vec![30u8; (w * h * 4) as usize];
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &init_pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wazoo cube uniforms"),
            size: std::mem::size_of::<CubeUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wazoo cube bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        CubeGpuEntry {
            texture,
            uniform_buffer,
            bind_group,
            width: w,
            height: h,
            alive,
            last_frame_seq: 0,
        }
    }
}

impl Pipeline for CubePipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wazoo cube shader module"),
            source: wgpu::ShaderSource::Wgsl(CUBE_SHADER_SRC.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wazoo cube bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("wazoo cube pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("wazoo cube render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Handled with discard in fragment shader
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("wazoo cube sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let texture_format = if format.is_srgb() {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };

        CubePipeline {
            render_pipeline,
            bind_group_layout,
            sampler,
            texture_format,
            entries: BTreeMap::new(),
        }
    }

    fn trim(&mut self) {
        self.entries
            .retain(|_, entry| entry.alive.load(Ordering::SeqCst));
    }
}

/// Represents a single active 3D cube instance to be rendered in the current frame,
/// packaging spatial transform parameters, video frame buffers, and shader effect toggles.
#[derive(Debug, Clone)]
pub struct CubeInstance {
    pub cube_id: u64,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
    pub edge_color: [f32; 4],
    pub player_id: u64,
    pub frame: Arc<Mutex<FrameData>>,
    pub alive: Arc<AtomicBool>,
    pub crt_enabled: bool,
    pub sheen_enabled: bool,
    pub opacity: f32,
}

/// Primitive shader bundle containing all active 3D cube instances for WGPU dispatch.
#[derive(Debug, Clone)]
pub struct CubePrimitive {
    pub instances: Vec<CubeInstance>,
}

impl Primitive for CubePrimitive {
    type Pipeline = CubePipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        let time_secs = (START_TIME.elapsed().as_secs_f64() % 3600.0) as f32;

        for inst in &self.instances {
            let mut frame_guard = inst.frame.lock().unwrap();
            let raw_width = frame_guard.width;
            let raw_height = frame_guard.height;
            let expected_len = (raw_width as usize) * (raw_height as usize) * 4;
            let has_valid_pixels = raw_width > 0 && raw_height > 0 && frame_guard.pixels.len() >= expected_len;

            let (width, height) = if has_valid_pixels {
                (raw_width, raw_height)
            } else {
                (128, 128)
            };

            let mut just_created = false;
            let needs_recreate = match pipeline.entries.get(&inst.cube_id) {
                Some(entry) => entry.width != width || entry.height != height,
                None => true,
            };

            if needs_recreate {
                let new_entry = pipeline.create_entry(
                    device,
                    queue,
                    width,
                    height,
                    Arc::clone(&inst.alive),
                );
                pipeline.entries.insert(inst.cube_id, new_entry);
                just_created = true;
            } else if let Some(entry) = pipeline.entries.get_mut(&inst.cube_id) {
                entry.alive = Arc::clone(&inst.alive);
            }

            let entry = pipeline.entries.get_mut(&inst.cube_id).unwrap();

            let is_new_frame = just_created
                || entry.last_frame_seq != frame_guard.frame_seq;

            if is_new_frame && has_valid_pixels {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &entry.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &frame_guard.pixels[..expected_len],
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(width * 4),
                        rows_per_image: Some(height),
                    },
                    wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
                entry.last_frame_seq = frame_guard.frame_seq;
                frame_guard.new_frame = false;
            }

            let uniforms = CubeUniforms {
                screen_size: [bounds.width.max(1.0), bounds.height.max(1.0)],
                cube_pos: [inst.x, inst.y],
                cube_size: inst.size.max(10.0),
                time: time_secs,
                _pad0: [0.0, 0.0],
                rotation: [inst.rx, inst.ry, inst.rz],
                has_texture: if has_valid_pixels { 1.0 } else { 0.0 },
                edge_color: inst.edge_color,
                opacity: inst.opacity.clamp(0.0, 1.0),
                crt_enabled: if inst.crt_enabled { 1.0 } else { 0.0 },
                sheen_enabled: if inst.sheen_enabled { 1.0 } else { 0.0 },
                _pad1: 0.0,
            };

            queue.write_buffer(&entry.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        }
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        let mut drawn = false;
        for inst in &self.instances {
            if let Some(entry) = pipeline.entries.get(&inst.cube_id) {
                render_pass.set_pipeline(&pipeline.render_pipeline);
                render_pass.set_bind_group(0, &entry.bind_group, &[]);
                render_pass.draw(0..36, 0..1);
                drawn = true;
            }
        }
        drawn
    }
}

/// Bridges iced's custom shader widget framework with the 3D cube rendering pipeline.
pub struct CubeProgram {
    instances: Vec<CubeInstance>,
}

impl CubeProgram {
    /// Creates a new `CubeProgram` with the specified collection of cube instances.
    pub fn new(instances: Vec<CubeInstance>) -> Self {
        Self { instances }
    }
}

impl<Message> Program<Message> for CubeProgram {
    type State = ();
    type Primitive = CubePrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        CubePrimitive {
            instances: self.instances.clone(),
        }
    }
}

/// Creates a full-window iced `Shader` element that renders all active 3D bouncing cubes.
pub fn cube_shader<Message>(program: CubeProgram) -> Shader<Message, CubeProgram> {
    Shader::new(program)
        .width(Length::Fill)
        .height(Length::Fill)
}
