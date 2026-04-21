import { spawn, execFileSync, ChildProcess } from 'child_process';
import ffmpegPathStatic from 'ffmpeg-static';
import * as ffprobePathStatic from 'ffprobe-static';

class FFmpegManager {
  private processes = new Map<string, ChildProcess>();

  public getFfmpegPath(): string {
    if (!ffmpegPathStatic) return 'ffmpeg';

    // In production, the binary is inside the app.asar, which can't be executed directly.
    // Electron Forge's asar.unpack configuration ensures it's available in app.asar.unpacked.
    if (ffmpegPathStatic.includes('app.asar.unpacked')) return ffmpegPathStatic;
    return ffmpegPathStatic.replace('app.asar', 'app.asar.unpacked');
  }

  public getFfprobePath(): string {
    const path = ffprobePathStatic?.path || (ffprobePathStatic as any)?.default?.path;
    if (!path) return 'ffprobe';

    // In production, the binary is inside the app.asar, which can't be executed directly.
    // Electron Forge's asar.unpack configuration ensures it's available in app.asar.unpacked.
    if (path.includes('app.asar.unpacked')) return path;
    return path.replace('app.asar', 'app.asar.unpacked');
  }

  public hasNvenc(): boolean {
    try {
      const ffmpegPath = this.getFfmpegPath();
      const output = execFileSync(ffmpegPath, ['-encoders'], { encoding: 'utf-8' });
      return output.includes('av1_nvenc');
    } catch (error) {
      console.error('Failed to check for NVENC support:', error);
      return false;
    }
  }

  public startLiveStream(id: string, filePath: string, start: string): ChildProcess {
    const ffmpegPath = this.getFfmpegPath();
    const args = [
      '-ss', start.toString(),
      '-i', filePath,
      '-c:v', 'libx264',
      '-preset', 'ultrafast',
      '-crf', '23',
      '-c:a', 'aac',
      '-b:a', '192k',
      '-movflags', 'frag_keyframe+empty_moov',
      '-f', 'mp4',
      'pipe:1'
    ];

    const child = spawn(ffmpegPath, args);
    this.processes.set(id, child);

    child.on('close', () => {
      this.processes.delete(id);
    });

    return child;
  }

  public startTranscode(id: string, filePath: string, outputPath: string, onStderr?: (data: string) => void): ChildProcess {
    const ffmpegPath = this.getFfmpegPath();
    const useNvenc = this.hasNvenc();

    const args = useNvenc ? [
      '-i', filePath,
      '-c:v', 'av1_nvenc',
      '-preset', 'p7',
      '-cq:v', '32',
      '-rc-lookahead', '32',
      '-spatial-aq', '1',
      '-temporal-aq', '1',
      '-c:a', 'copy',
      '-map', '0',
      '-y', outputPath
    ] : [
      '-i', filePath,
      '-c:v', 'libx264',
      '-preset', 'fast',
      '-crf', '23',
      '-c:a', 'copy',
      '-map', '0',
      '-y', outputPath
    ];

    const child = spawn(ffmpegPath, args);
    this.processes.set(id, child);

    if (onStderr && child.stderr) {
      child.stderr.on('data', (data) => {
        onStderr(data.toString());
      });
    }

    child.on('close', () => {
      this.processes.delete(id);
    });

    return child;
  }

  public killProcess(id: string): void {
    const child = this.processes.get(id);
    if (child) {
      child.kill('SIGKILL');
      this.processes.delete(id);
    }
  }

  public killAll(): void {
    for (const [id, child] of this.processes.entries()) {
      child.kill('SIGKILL');
      this.processes.delete(id);
    }
  }

  public async extractSubtitles(inputPath: string, outputPath: string): Promise<void> {
    const ffmpegPath = this.getFfmpegPath();
    // Extract the first subtitle stream (index 0 of subtitle streams)
    const args = [
      '-i', inputPath,
      '-map', '0:s:0',
      '-y', outputPath
    ];

    return new Promise((resolve, reject) => {
      const child = spawn(ffmpegPath, args);
      
      child.on('close', (code) => {
        if (code === 0) resolve();
        else reject(new Error(`FFmpeg exited with code ${code}`));
      });

      child.on('error', (err) => {
        reject(err);
      });
    });
  }
}

export const ffmpegManager = new FFmpegManager();

