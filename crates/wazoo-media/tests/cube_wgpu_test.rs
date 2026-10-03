use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use iced::advanced::graphics::Viewport;
use iced::widget::shader::{Pipeline, Primitive};
use iced::Rectangle;
use wazoo_media::cube::{CubeInstance, CubePipeline, CubePrimitive};
use wazoo_media::pipeline::FrameData;

#[tokio::test]
async fn test_cube_pipeline_creation_and_render() {
    let instance = iced_wgpu::wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&iced_wgpu::wgpu::RequestAdapterOptions {
            power_preference: iced_wgpu::wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await;

    let adapter = match adapter {
        Ok(a) => a,
        Err(e) => {
            eprintln!("No adapter found ({:?}), skipping headless test", e);
            return;
        }
    };

    let (device, queue) = adapter
        .request_device(&iced_wgpu::wgpu::DeviceDescriptor::default())
        .await
        .expect("Failed to create device");

    let format = iced_wgpu::wgpu::TextureFormat::Bgra8UnormSrgb;
    let mut pipeline = CubePipeline::new(&device, &queue, format);

    let frame = Arc::new(Mutex::new(FrameData {
        width: 64,
        height: 64,
        pixels: vec![200u8; 64 * 64 * 4],
        new_frame: true,
        frame_seq: 1,
    }));
    let alive = Arc::new(AtomicBool::new(true));

    let inst = CubeInstance {
        cube_id: 1,
        x: 400.0,
        y: 300.0,
        size: 80.0,
        rx: 0.5,
        ry: 0.8,
        rz: 0.2,
        edge_color: [0.26, 0.72, 0.51, 1.0],
        player_id: 1,
        frame,
        alive,
        crt_enabled: false,
        sheen_enabled: false,
        opacity: 1.0,
    };

    let primitive = CubePrimitive {
        instances: vec![inst],
    };

    let bounds = Rectangle::new(iced::Point::ORIGIN, iced::Size::new(800.0, 600.0));
    let viewport = Viewport::with_physical_size(
        iced::Size::new(800, 600),
        1.0,
    );

    primitive.prepare(&mut pipeline, &device, &queue, &bounds, &viewport);

    // Create a target texture to render into
    let target = device.create_texture(&iced_wgpu::wgpu::TextureDescriptor {
        label: Some("test target"),
        size: iced_wgpu::wgpu::Extent3d {
            width: 800,
            height: 600,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: iced_wgpu::wgpu::TextureDimension::D2,
        format,
        usage: iced_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let target_view = target.create_view(&iced_wgpu::wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&iced_wgpu::wgpu::CommandEncoderDescriptor::default());
    {
        let mut render_pass = encoder.begin_render_pass(&iced_wgpu::wgpu::RenderPassDescriptor {
            label: Some("test pass"),
            color_attachments: &[Some(iced_wgpu::wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: iced_wgpu::wgpu::Operations {
                    load: iced_wgpu::wgpu::LoadOp::Clear(iced_wgpu::wgpu::Color::BLACK),
                    store: iced_wgpu::wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let drawn = primitive.draw(&pipeline, &mut render_pass);
        assert!(drawn);
    }
    queue.submit(Some(encoder.finish()));
    println!("SUCCESSFULLY EXECUTED WGPU CUBE PIPELINE!");
}
