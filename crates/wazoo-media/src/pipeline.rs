/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Demuxing & Decoding Pipeline
 *
 * Manages low-level video stream decoding, frame rate tracking, buffer status monitoring,
 * and format negotiation for media playback.
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    scale: [f32; 2],
    opacity: f32,
    crt_enabled: f32,
    resolution: [f32; 2],
    video_res: [f32; 2],
    time: f32,
    wavy_enabled: f32,
    fog_enabled: f32,
    _pad: f32,
}

pub struct FrameData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub new_frame: bool,
    pub frame_seq: u64,
}

impl std::fmt::Debug for FrameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameData")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pixels_len", &self.pixels.len())
            .field("new_frame", &self.new_frame)
            .field("frame_seq", &self.frame_seq)
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
    last_frame_seq: u64,
}

pub struct VideoPipeline {
    default_pipeline: wgpu::RenderPipeline,
    crt_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    texture_format: wgpu::TextureFormat,
    videos: BTreeMap<u64, VideoEntry>,
}

const DEFAULT_SHADER_SRC: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct Uniforms {
    scale: vec2<f32>,
    opacity: f32,
    crt_enabled: f32,
    resolution: vec2<f32>,
    video_res: vec2<f32>,
    time: f32,
    wavy_enabled: f32,
    fog_enabled: f32,
    _pad: f32,
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
    let raw_uv = uv[in_vertex_index];
    out.uv = (raw_uv - vec2<f32>(0.5, 0.5)) / uniforms.scale + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.uv.x < 0.0 || in.uv.x > 1.0 || in.uv.y < 0.0 || in.uv.y > 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, uniforms.opacity);
    }
    var col = textureSampleLevel(tex, s, in.uv, 0.0);
    return vec4<f32>(col.rgb, uniforms.opacity);
}
"#;

const CRT_SHADER_SRC: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct Uniforms {
    scale: vec2<f32>,
    opacity: f32,
    crt_enabled: f32,
    resolution: vec2<f32>,
    video_res: vec2<f32>,
    time: f32,
    wavy_enabled: f32,
    fog_enabled: f32,
    _pad: f32,
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
    let raw_uv = uv[in_vertex_index];
    out.uv = (raw_uv - vec2<f32>(0.5, 0.5)) / uniforms.scale + vec2<f32>(0.5, 0.5);
    return out;
}

fn curve(uv: vec2<f32>, curvature: f32) -> vec2<f32> {
    var u = (uv - 0.5) * 2.0;
    u.x = u.x * (1.0 + pow(abs(u.y) / curvature, 2.0));
    u.y = u.y * (1.0 + pow(abs(u.x) / curvature, 2.0));
    return (u / 2.0) + 0.5;
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn hash_fog(p_in: vec2<f32>) -> f32 {
    var p = fract(p_in * vec2<f32>(5.3983, 5.4427));
    p += vec2<f32>(dot(p.yx, p + vec2<f32>(19.19, 19.19)));
    return fract(p.x * p.y);
}

fn noise2d_fog(p_in: vec2<f32>) -> f32 {
    var p = p_in + vec2<f32>(8192.0, 8192.0);
    var i = floor(p);
    var f = fract(p);
    var u = f * f * (3.0 - 2.0 * f);

    var a = hash_fog(i);
    var b = hash_fog(i + vec2<f32>(1.0, 0.0));
    var c = hash_fog(i + vec2<f32>(0.0, 1.0));
    var d = hash_fog(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm_fog(uv: vec2<f32>, time: f32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var drift1 = vec2<f32>(time * 0.04, time * 0.02);
    var drift2 = vec2<f32>(-time * 0.03, time * 0.05);
    var drift3 = vec2<f32>(time * 0.06, -time * 0.04);

    value += amplitude * noise2d_fog(uv * frequency + drift1);
    frequency *= 2.0;
    amplitude *= 0.5;
    value += amplitude * noise2d_fog(uv * frequency + drift2);
    frequency *= 2.0;
    amplitude *= 0.5;
    value += amplitude * noise2d_fog(uv * frequency + drift3);
    return value;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var base_color: vec4<f32>;
    var rgb: vec3<f32>;

    // 1. CRT filter (curvature, tracking distortion, chromatic aberration, scanlines)
    if (uniforms.crt_enabled > 0.5) {
        let curved_uv = curve(in.uv, 3.2);

        // Strict letterbox / pillarbox check for CRT mode
        if curved_uv.x < 0.0 || curved_uv.x > 1.0 || curved_uv.y < 0.0 || curved_uv.y > 1.0 {
            return vec4<f32>(0.0, 0.0, 0.0, uniforms.opacity);
        }

        // Smooth rounded CRT glass envelope corners
        let corner_box = abs(curved_uv - vec2<f32>(0.5, 0.5)) * 2.0;
        let corner_dist = pow(corner_box.x, 10.0) + pow(corner_box.y, 10.0);
        if corner_dist > 1.05 {
            return vec4<f32>(0.0, 0.0, 0.0, uniforms.opacity);
        }

        // Apply wavy fluid displacement within the curved CRT tube
        var screen_uv = curved_uv;
        if (uniforms.wavy_enabled > 0.5) {
            let speed = 0.7;
            let freq = 16.0;
            let amp = 0.004;
            var wave_x = sin(curved_uv.y * freq + uniforms.time * speed) * amp;
            var wave_y = cos(curved_uv.x * freq + uniforms.time * speed * 0.8) * amp;
            wave_x += sin(curved_uv.x * (freq * 0.5) - uniforms.time * (speed * 0.6)) * (amp * 0.4);
            wave_y += cos(curved_uv.y * (freq * 0.5) + uniforms.time * (speed * 0.7)) * (amp * 0.4);
            screen_uv = clamp(curved_uv + vec2<f32>(wave_x, wave_y), vec2<f32>(0.0), vec2<f32>(1.0));
        }

        // Signal glitches / VHS tracking distortion
        let time_step = floor(uniforms.time * 6.0);
        let glitch_chance = hash(vec2<f32>(time_step, 17.0));

        var glitch_offset = 0.0;
        var shift = 0.0018;

        if glitch_chance > 0.84 {
            let band_start = hash(vec2<f32>(time_step, 42.0));
            let band_end = band_start + 0.02 + hash(vec2<f32>(time_step, 99.0)) * 0.08;

            if screen_uv.y >= band_start && screen_uv.y <= band_end {
                glitch_offset = (hash(vec2<f32>(time_step, floor(screen_uv.y * 50.0))) - 0.5) * 0.004;
                shift += 0.0045 * hash(vec2<f32>(time_step, 88.0));
            }

            let roll = fract(uniforms.time * 0.2);
            let roll_dist = abs(screen_uv.y - roll);
            if roll_dist < 0.04 {
                glitch_offset += sin((screen_uv.y - roll) * 50.0) * 0.0015;
                shift += 0.0025;
            }
        }

        let warped_uv = clamp(vec2<f32>(screen_uv.x + glitch_offset, screen_uv.y), vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));

        // Radial chromatic aberration
        let center_dir = warped_uv - vec2<f32>(0.5, 0.5);
        let dist_sq = dot(center_dir, center_dir);
        let ca_radial = center_dir * (dist_sq * 0.045);
        let ca_offset = ca_radial + vec2<f32>(shift, 0.0);

        let uv_r = clamp(warped_uv + ca_offset, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
        let uv_g = warped_uv;
        let uv_b = clamp(warped_uv - ca_offset, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));

        let r = textureSampleLevel(tex, s, uv_r, 0.0).r;
        let g = textureSampleLevel(tex, s, uv_g, 0.0).g;
        let b = textureSampleLevel(tex, s, uv_b, 0.0).b;
        let a = textureSampleLevel(tex, s, warped_uv, 0.0).a;
        base_color = vec4<f32>(r, g, b, a);

        // CRT phosphor luminescence & gamma compensation:
        // Automatically compensates for the light loss introduced by scanlines
        //let clamped_rgb = clamp(base_color.rgb, vec3<f32>(0.00001), vec3<f32>(1.0));
        //let phosphor_bright = pow(clamped_rgb, vec3<f32>(0.90)) * 1.10;
        // Active rolling cathode scanlines (multiplicative modulation avoids shadow crush)
        //let scanline = sin(curved_uv.y * 600.0 + uniforms.time * 5.0) * 0.5 + 0.5;
        //let scanline_factor = 1.0 - scanline * 0.16;
        //rgb = phosphor_bright * scanline_factor;

        // Active rolling cathode scanlines
        let scanline = sin(curved_uv.y * 600.0 + uniforms.time * 5.0) * 0.5 + 0.5;
        rgb = base_color.rgb - scanline * 0.20;

        // Animated RF static noise
        let noise = (hash(curved_uv + vec2<f32>(uniforms.time, uniforms.time)) - 0.5) * 0.016;
        rgb += vec3<f32>(noise, noise, noise);

        // CRT bezel falloff vignette
        let vignette = curved_uv.x * curved_uv.y * (1.0 - curved_uv.x) * (1.0 - curved_uv.y);
        let vig_factor = clamp(pow(16.0 * vignette, 0.28), 0.0, 1.0);
        rgb *= vig_factor;
    } else {
        // CRT is disabled: ensure pillarbox/letterbox black bars are preserved and never smeared
        if in.uv.x < 0.0 || in.uv.x > 1.0 || in.uv.y < 0.0 || in.uv.y > 1.0 {
            return vec4<f32>(0.0, 0.0, 0.0, uniforms.opacity);
        }

        // Apply wavy fluid displacement strictly inside the video frame
        var screen_uv = in.uv;
        if (uniforms.wavy_enabled > 0.5) {
            let speed = 0.7;
            let freq = 16.0;
            let amp = 0.004;
            var wave_x = sin(in.uv.y * freq + uniforms.time * speed) * amp;
            var wave_y = cos(in.uv.x * freq + uniforms.time * speed * 0.8) * amp;
            wave_x += sin(in.uv.x * (freq * 0.5) - uniforms.time * (speed * 0.6)) * (amp * 0.4);
            wave_y += cos(in.uv.y * (freq * 0.5) + uniforms.time * (speed * 0.7)) * (amp * 0.4);
            screen_uv = clamp(in.uv + vec2<f32>(wave_x, wave_y), vec2<f32>(0.0), vec2<f32>(1.0));
        }

        base_color = textureSampleLevel(tex, s, screen_uv, 0.0);
        rgb = base_color.rgb;
    }

    // 3. Volumetric Fog filter (atmospheric rolling mist with transparent clearings)
    if (uniforms.fog_enabled > 0.5) {
        let fog_uv = in.uv * (uniforms.resolution / 750.0);
        let shadow_color = vec3<f32>(0.24, 0.28, 0.35);
        let light_color = vec3<f32>(0.72, 0.76, 0.82);
        let fog_intensity = 0.28;

        var q = vec2<f32>(
            fbm_fog(fog_uv * 1.8 + vec2<f32>(uniforms.time * 0.08, uniforms.time * 0.05), uniforms.time),
            fbm_fog(fog_uv * 1.8 + vec2<f32>(-uniforms.time * 0.06, uniforms.time * 0.10), uniforms.time)
        );

        var r = vec2<f32>(
            fbm_fog(fog_uv * 2.5 + q * 1.8 + vec2<f32>(uniforms.time * 0.07, -uniforms.time * 0.05), uniforms.time),
            fbm_fog(fog_uv * 2.5 + q * 2.2 + vec2<f32>(-uniforms.time * 0.08, uniforms.time * 0.09), uniforms.time)
        );

        var f = fbm_fog(fog_uv * 2.0 + r * 2.2, uniforms.time);
        var ridged_fog = 1.0 - abs(f - 0.5) * 2.0;
        var combined = mix(f, ridged_fog, 0.30);
        var density = smoothstep(0.42, 0.82, combined);

        var eps = 0.018;
        var f_offset = fbm_fog((fog_uv + vec2<f32>(eps, eps)) * 2.0 + r * 2.2, uniforms.time);
        var diff = clamp(f - f_offset, 0.0, 1.0);

        var fog_color = mix(shadow_color, light_color, density);
        var edge_highlight = smoothstep(0.1, 0.5, density) * (1.0 - smoothstep(0.4, 0.8, density));
        edge_highlight *= diff * 3.5;
        fog_color += vec3<f32>(0.35, 0.42, 0.52) * edge_highlight;

        var final_fog = density * fog_intensity;
        rgb = mix(rgb, fog_color, final_fog);
    }

    return vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), uniforms.opacity);
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
            last_frame_seq: 0,
        }
    }
}

impl Pipeline for VideoPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let default_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wazoo default video shader module"),
            source: wgpu::ShaderSource::Wgsl(DEFAULT_SHADER_SRC.into()),
        });

        let crt_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wazoo crt video shader module"),
            source: wgpu::ShaderSource::Wgsl(CRT_SHADER_SRC.into()),
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
            label: Some("wazoo video pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let default_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("wazoo default video render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &default_shader,
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
                module: &default_shader,
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

        let crt_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("wazoo crt video render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &crt_shader,
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
                module: &crt_shader,
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
            default_pipeline,
            crt_pipeline,
            bind_group_layout,
            sampler,
            texture_format,
            videos: BTreeMap::new(),
        }
    }

    fn trim(&mut self) {
        self.videos
            .retain(|_, entry| entry.alive.load(Ordering::SeqCst));
    }
}

#[derive(Debug, Clone)]
pub struct VideoPrimitive {
    player_id: u64,
    frame: Arc<Mutex<FrameData>>,
    alive: Arc<AtomicBool>,
    opacity: f32,
    fit_cover: bool,
    crt_enabled: bool,
    wavy_enabled: bool,
    fog_enabled: bool,
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

        let mut just_created = false;
        let needs_recreate = match pipeline.videos.get(&self.player_id) {
            Some(entry) => entry.width != width || entry.height != height,
            None => true,
        };

        if needs_recreate {
            let new_entry = pipeline.create_entry(device, width, height, Arc::clone(&self.alive));
            pipeline.videos.insert(self.player_id, new_entry);
            just_created = true;
        } else {
            let entry = pipeline.videos.get_mut(&self.player_id).unwrap();
            entry.alive = Arc::clone(&self.alive);
        }

        let entry = pipeline.videos.get_mut(&self.player_id).unwrap();

        let is_new_frame =
            just_created || entry.last_frame_seq != frame_guard.frame_seq;

        if is_new_frame && !frame_guard.pixels.is_empty() {
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
            entry.last_frame_seq = frame_guard.frame_seq;
            frame_guard.new_frame = false;
        }

        let (vw, vh) = (width as f32, height as f32);
        let (bw, bh) = (bounds.width, bounds.height);
        let (scale_x, scale_y) = if bw > 0.0 && bh > 0.0 && vw > 0.0 && vh > 0.0 {
            let video_aspect = vw / vh;
            let bounds_aspect = bw / bh;
            if self.fit_cover {
                // object-fit: cover (zooms/crops to fill entire bounds seamlessly)
                if bounds_aspect > video_aspect {
                    (1.0, bounds_aspect / video_aspect)
                } else {
                    (video_aspect / bounds_aspect, 1.0)
                }
            } else {
                // object-fit: contain (letterbox / pillarbox with black bars)
                if bounds_aspect > video_aspect {
                    (video_aspect / bounds_aspect, 1.0)
                } else {
                    (1.0, bounds_aspect / video_aspect)
                }
            }
        } else {
            (1.0, 1.0)
        };

        let time_secs = (START_TIME.elapsed().as_secs_f64() % 3600.0) as f32;

        let uniforms = Uniforms {
            scale: [scale_x, scale_y],
            opacity: self.opacity,
            crt_enabled: if self.crt_enabled { 1.0 } else { 0.0 },
            resolution: [bw.max(1.0), bh.max(1.0)],
            video_res: [vw.max(1.0), vh.max(1.0)],
            time: time_secs,
            wavy_enabled: if self.wavy_enabled { 1.0 } else { 0.0 },
            fog_enabled: if self.fog_enabled { 1.0 } else { 0.0 },
            _pad: 0.0,
        };
        queue.write_buffer(&entry.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        if let Some(entry) = pipeline.videos.get(&self.player_id) {
            let active_pipeline = if self.crt_enabled || self.wavy_enabled || self.fog_enabled {
                &pipeline.crt_pipeline
            } else {
                &pipeline.default_pipeline
            };
            render_pass.set_pipeline(active_pipeline);
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
    opacity: f32,
    fit_cover: bool,
    crt_enabled: bool,
    wavy_enabled: bool,
    fog_enabled: bool,
}

impl VideoProgram {
    pub fn new(
        player_id: u64,
        frame: Arc<Mutex<FrameData>>,
        alive: Arc<AtomicBool>,
        opacity: f32,
    ) -> Self {
        Self::new_with_fit(player_id, frame, alive, opacity, false)
    }

    pub fn new_with_fit(
        player_id: u64,
        frame: Arc<Mutex<FrameData>>,
        alive: Arc<AtomicBool>,
        opacity: f32,
        fit_cover: bool,
    ) -> Self {
        Self::new_full(
            player_id, frame, alive, opacity, fit_cover, false, false, false,
        )
    }

    pub fn new_full(
        player_id: u64,
        frame: Arc<Mutex<FrameData>>,
        alive: Arc<AtomicBool>,
        opacity: f32,
        fit_cover: bool,
        crt_enabled: bool,
        wavy_enabled: bool,
        fog_enabled: bool,
    ) -> Self {
        Self {
            player_id,
            frame,
            alive,
            opacity,
            fit_cover,
            crt_enabled,
            wavy_enabled,
            fog_enabled,
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
            opacity: self.opacity,
            fit_cover: self.fit_cover,
            crt_enabled: self.crt_enabled,
            wavy_enabled: self.wavy_enabled,
            fog_enabled: self.fog_enabled,
        }
    }
}

pub fn video_shader<Message>(program: VideoProgram) -> Shader<Message, VideoProgram> {
    Shader::new(program)
        .width(Length::Fill)
        .height(Length::Fill)
}
