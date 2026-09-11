use std::path::Path;
use wazoo_media::VideoHandle;

#[test]
fn test_video_handle_gstreamer() {
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let handle = VideoHandle::new(1, sample, "Test Video");
        assert!(handle.is_ok(), "Failed to create VideoHandle: {:?}", handle.err());
        let mut handle = handle.unwrap();

        let dur = handle.duration();
        println!("GStreamer pipeline initialized. Reported duration: {:?}", dur);
        assert_eq!(handle.state.volume, 1.0);

        handle.set_volume(0.5);
        assert_eq!(handle.state.volume, 0.5);

        handle.set_muted(true);
        assert!(handle.state.is_muted);
    }
}

#[test]
fn test_pipeline_buffering_and_queue_properties() {
    use gstreamer as gst;
    use gstreamer::prelude::*;

    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let handle = VideoHandle::new(2, sample, "Buffer Test Video").unwrap();
        let bin = handle.pipeline.upcast::<gst::Bin>();

        // Check playbin properties
        let buf_dur: i64 = bin.property("buffer-duration");
        let buf_size: i32 = bin.property("buffer-size");
        let ring_buf: u64 = bin.property("ring-buffer-max-size");
        println!("playbin buffer-duration: {buf_dur}, buffer-size: {buf_size}, ring-buffer: {ring_buf}");
        assert_eq!(buf_dur, 10_000_000_000);
        assert_eq!(buf_size, 64 * 1024 * 1024);
        assert_eq!(ring_buf, 64 * 1024 * 1024);

        // Find multiqueue inside decodebin
        let mut found_multiqueue = false;
        let mut found_source = false;

        let mut it = bin.iterate_elements();
        while let Ok(Some(elem)) = it.next() {
            if elem.name().starts_with("uridecodebin") {
                if let Ok(uri_bin) = elem.downcast::<gst::Bin>() {
                    let mut it2 = uri_bin.iterate_elements();
                    while let Ok(Some(elem2)) = it2.next() {
                        if elem2.name().starts_with("source") {
                            if elem2.has_property("blocksize", None) {
                                let bs: u32 = elem2.property("blocksize");
                                println!("source blocksize: {bs}");
                                assert_eq!(bs, 524_288);
                                found_source = true;
                            }
                        } else if elem2.name().starts_with("decodebin") {
                            if let Ok(dec_bin) = elem2.downcast::<gst::Bin>() {
                                let mut it3 = dec_bin.iterate_elements();
                                while let Ok(Some(elem3)) = it3.next() {
                                    if elem3.name().starts_with("multiqueue") {
                                        let max_buffers: u32 = elem3.property("max-size-buffers");
                                        let max_time: u64 = elem3.property("max-size-time");
                                        let max_bytes: u32 = elem3.property("max-size-bytes");
                                        println!("multiqueue ({}): buffers={max_buffers}, time={max_time}, bytes={max_bytes}", elem3.name());
                                        assert!(max_buffers >= 1000, "Expected multiqueue buffer limit >= 1000, got {max_buffers}");
                                        assert_eq!(max_time, 10_000_000_000);
                                        assert_eq!(max_bytes, 128 * 1024 * 1024);
                                        found_multiqueue = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check vqueue and aqueue
        let mut found_vqueue = false;
        let mut found_aqueue = false;
        let mut it_playsink = bin.iterate_elements();
        while let Ok(Some(elem)) = it_playsink.next() {
            if elem.name().starts_with("playsink") {
                if let Ok(playsink_bin) = elem.downcast::<gst::Bin>() {
                    let mut it_play = playsink_bin.iterate_elements();
                    while let Ok(Some(child)) = it_play.next() {
                        if child.name().starts_with("vbin") {
                            if let Ok(vbin) = child.downcast::<gst::Bin>() {
                                if let Some(vq) = vbin.by_name("vqueue") {
                                    let vq_buf: u32 = vq.property("max-size-buffers");
                                    let vq_time: u64 = vq.property("max-size-time");
                                    println!("vqueue: buffers={vq_buf}, time={vq_time}");
                                    assert_eq!(vq_buf, 60);
                                    assert_eq!(vq_time, 1_000_000_000);
                                    found_vqueue = true;
                                }
                            }
                        } else if child.name().starts_with("abin") {
                            if let Ok(abin) = child.downcast::<gst::Bin>() {
                                if let Some(aq) = abin.by_name("aqueue") {
                                    let aq_time: u64 = aq.property("max-size-time");
                                    println!("aqueue: time={aq_time}");
                                    assert_eq!(aq_time, 2_000_000_000);
                                    found_aqueue = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        assert!(found_source, "source element with blocksize was not found");
        assert!(found_multiqueue, "multiqueue element was not found or configured");
        assert!(found_vqueue, "vqueue element was not found or configured");
        assert!(found_aqueue, "aqueue element was not found or configured");
    }
}

#[test]
fn test_seek_behavior() {
    use std::time::Duration;
    let sample = "/home/klo/Downloads/VID_20240309_123459_679.mp4";
    if Path::new(sample).exists() {
        let mut handle = VideoHandle::new(1, sample, "Test Video").unwrap();
        handle.set_muted(true);
        handle.play();
        std::thread::sleep(Duration::from_millis(500));
        let pos1 = handle.position();
        println!("pos1 (initial ~500ms): {:?}", pos1);
        assert!(pos1 > Duration::ZERO);

        // First seek +5.0s
        handle.seek_relative(5.0);
        let pos_after_seek1 = handle.position();
        println!("pos immediately after +5.0s: {:?}", pos_after_seek1);
        // It must NOT be 0s! It must be near 5.5s
        assert!(pos_after_seek1 >= Duration::from_secs(5), "Expected >= 5s, got {:?}", pos_after_seek1);

        // Immediate second seek +3.0s (accumulates)
        handle.seek_relative(3.0);
        let pos_after_seek2 = handle.position();
        println!("pos immediately after second +3.0s: {:?}", pos_after_seek2);
        assert!(pos_after_seek2 >= Duration::from_secs(8), "Expected >= 8s, got {:?}", pos_after_seek2);

        // Seek backward -5.0s (should be around 3.5s, NOT 0s!)
        handle.seek_relative(-5.0);
        let pos_after_seek3 = handle.position();
        println!("pos immediately after -5.0s: {:?}", pos_after_seek3);
        assert!(pos_after_seek3 >= Duration::from_secs(2), "Expected >= 2s, got {:?}", pos_after_seek3);
        assert!(pos_after_seek3 < Duration::from_secs(6), "Expected < 6s, got {:?}", pos_after_seek3);

        // Seek backward past 0 (should clamp to 0)
        handle.seek_relative(-10.0);
        let pos_clamped = handle.position();
        println!("pos after -10.0s (clamped): {:?}", pos_clamped);
        assert_eq!(pos_clamped, Duration::ZERO);
    }
}

