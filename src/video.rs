use std::sync::OnceLock;
use ffmpeg_next as ffmpeg;

pub fn video_formats() -> &'static Vec<&'static str> {
    static VIDEO_FORMATS: OnceLock<Vec<&'static str>> = OnceLock::new();
    VIDEO_FORMATS.get_or_init(|| {
        vec!["mp4", "mkv"]
    })
}

pub fn get_video_length(file_path: &str) -> Result<u64, Box<dyn std::error::Error>> {
    ffmpeg::init()?;

    // Open the input video file
    let context = ffmpeg::format::input(&file_path)?;

    // Get the duration of the video in microseconds
    let duration = context.duration();

    if duration > 0 {
        Ok((duration / ffmpeg::ffi::AV_TIME_BASE as i64) as u64)
    } else {
        Err("Failed to get video duration".into())
    }
}