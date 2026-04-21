#!/bin/bash
# Smart media transcoder: Skips supported formats, prefers av1_nvenc, falls back to libx264.

if [ -z "$1" ]; then
  echo "Usage: ./lorebase-encode.sh <video_file_or_directory>"
  exit 1
fi

# 1. Check if the local FFmpeg build has Nvidia AV1 hardware encoding
HAS_NVENC=false
if ffmpeg -encoders 2>/dev/null | grep -q "av1_nvenc"; then
  HAS_NVENC=true
fi

process_file() {
  local input_file="$1"
  local filename=$(basename -- "$input_file")
  local base="${filename%.*}"
  local output_file="${base}_lorebase.mkv"

  # 2. Extract the exact video codec using ffprobe
  local codec
  codec=$(ffprobe -v error -select_streams v:0 -show_entries stream=codec_name -of default=noprint_wrappers=1:nokey=1 "$input_file")

  if [ -z "$codec" ]; then
    echo "⚠️  Could not detect codec for $filename. Skipping."
    return
  fi

  # 3. Skip natively supported codecs (Chromium handles these perfectly without HEVC patent drama)
  if [[ "$codec" == "h264" || "$codec" == "av1" || "$codec" == "vp9" ]]; then
    echo "✅ Skipping $filename (Already uses supported codec: $codec)"
    return
  fi

  echo "🎬 Processing $filename (Detected unsupported codec: $codec)"

  # 4. Route to the correct encoder
  if [ "$HAS_NVENC" = true ]; then
    echo "🚀 Nvidia AV1 hardware encoder detected. Engaging av1_nvenc..."
    # Your exact AV1 NVENC string, keeping all streams (-map 0) and copying audio to prevent degradation
    ffmpeg -i "$input_file" -c:v av1_nvenc -preset p7 -cq:v 32 -rc-lookahead 32 -spatial-aq 1 -temporal-aq 1 -c:a copy -map 0 "$output_file"
  else
    echo "🐌 No AV1 hardware detected. Falling back to CPU H.264 (libx264)..."
    # CPU fallback for users without modern Nvidia GPUs
    ffmpeg -i "$input_file" -c:v libx264 -preset fast -crf 23 -c:a copy -map 0 "$output_file"
  fi

  echo "✨ Finished converting to $output_file"
}

# Handle both single files and directories
if [ -d "$1" ]; then
  for file in "$1"/*; do
    if [ -f "$file" ]; then
      process_file "$file"
    fi
  done
else
  process_file "$1"
fi
