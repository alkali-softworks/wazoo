use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bytemuck::{Pod, Zeroable};
use iced::advanced::graphics::Viewport;
use iced::widget::shader::{Pipeline, Primitive, Program, Shader};
use iced::{mouse, Length, Rectangle};
use iced_wgpu::wgpu;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    scale: [f32; 2],
    _pad: [f32; 2],
}

pub struct FrameData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub new_frame: bool,
}

impl std::fmt::Debug for FrameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameData")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pixels_len", &self.pixels.len())
            .field("new_frame", &self.new_frame)
            .finish()
    }
}

struct VideoEntry {
    texture: wgpu::Texture,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
    alive: Arc<AtomicBool>,
}

pub struct VideoPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    texture_format: wgpu::TextureFormat,
    videos: BTreeMap<u64, VideoEntry>,
}

const SHADER_SRC: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct Uniforms {
    scale: vec2<f32>,
    _pad: vec2<f32>,
}

@group(0) @binding(0)
var tex: texture_2d<f32>;

@group(0) @binding(1)
var s: sampler;

@group(0) @binding(2)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0, -1.0),
    );
    var uv = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );

    var out: VertexOutput;
    out.position = vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
    // Center video and scale inside viewport
    let raw_uv = uv[in_vertex_index];
    out.uv = (raw_uv - vec2<f32>(0.5, 0.5)) / uniforms.scale + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.uv.x < 0.0 || in.uv.x > 1.0 || in.uv.y < 0.0 || in.uv.y > 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return textureSample(tex, s, in.uv);
}
"#;

impl VideoPipeline {
    fn create_entry(
        &self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        alive: Arc<AtomicBool>,
    ) -> VideoEntry {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("wazoo video texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wazoo video uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wazoo video bind group"),
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

        VideoEntry {
            texture,
            uniform_buffer,
            bind_group,
            width,
            height,
            alive,
        }
    }
}

impl Pipeline for VideoPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wazoo video shader module"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wazoo video bind group layout"),
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
                    visibility: wgpu::ShaderStages::VERTEX,
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
            label: Some("wazoo video pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("wazoo video render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
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
            label: Some("wazoo video sampler"),
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

        VideoPipeline {
            pipeline,
            bind_group_layout,
            sampler,
            texture_format,
            videos: BTreeMap::new(),
        }
    }

    fn trim(&mut self) {
        self.videos.retain(|_, entry| entry.alive.load(Ordering::SeqCst));
    }
}

#[derive(Debug, Clone)]
pub struct VideoPrimitive {
    player_id: u64,
    frame: Arc<Mutex<FrameData>>,
    alive: Arc<AtomicBool>,
}

impl Primitive for VideoPrimitive {
    type Pipeline = VideoPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        let mut frame_guard = self.frame.lock().unwrap();
        let width = frame_guard.width;
        let height = frame_guard.height;

        if width == 0 || height == 0 {
            return;
        }

        let needs_recreate = match pipeline.videos.get(&self.player_id) {
            Some(entry) => entry.width != width || entry.height != height,
            None => true,
        };

        if needs_recreate {
            let new_entry = pipeline.create_entry(device, width, height, Arc::clone(&self.alive));
            pipeline.videos.insert(self.player_id, new_entry);
        }

        let entry = pipeline.videos.get_mut(&self.player_id).unwrap();

        if frame_guard.new_frame && !frame_guard.pixels.is_empty() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &entry.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &frame_guard.pixels,
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
            frame_guard.new_frame = false;
        }

        let (vw, vh) = (width as f32, height as f32);
        let (bw, bh) = (bounds.width, bounds.height);
        let (scale_x, scale_y) = if bw > 0.0 && bh > 0.0 && vw > 0.0 && vh > 0.0 {
            let video_aspect = vw / vh;
            let bounds_aspect = bw / bh;
            if bounds_aspect > video_aspect {
                (video_aspect / bounds_aspect, 1.0)
            } else {
                (1.0, bounds_aspect / video_aspect)
            }
        } else {
            (1.0, 1.0)
        };

        let uniforms = Uniforms {
            scale: [scale_x, scale_y],
            _pad: [0.0, 0.0],
        };
        queue.write_buffer(&entry.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    fn draw(
        &self,
        pipeline: &Self::Pipeline,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) -> bool {
        if let Some(entry) = pipeline.videos.get(&self.player_id) {
            render_pass.set_pipeline(&pipeline.pipeline);
            render_pass.set_bind_group(0, &entry.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
            return true;
        }
        false
    }
}

pub struct VideoProgram {
    player_id: u64,
    frame: Arc<Mutex<FrameData>>,
    alive: Arc<AtomicBool>,
}

impl VideoProgram {
    pub fn new(player_id: u64, frame: Arc<Mutex<FrameData>>, alive: Arc<AtomicBool>) -> Self {
        Self {
            player_id,
            frame,
            alive,
        }
    }
}

impl<Message> Program<Message> for VideoProgram {
    type State = ();
    type Primitive = VideoPrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        VideoPrimitive {
            player_id: self.player_id,
            frame: Arc::clone(&self.frame),
            alive: Arc::clone(&self.alive),
        }
    }
}

pub fn video_shader<Message>(program: VideoProgram) -> Shader<Message, VideoProgram> {
    Shader::new(program).width(Length::Fill).height(Length::Fill)
}
