import { Hono } from 'hono';
import { stream } from 'hono/streaming';
import { ffmpegManager } from '@/main/ffmpegManager';
import { BrowserWindow } from 'electron';
import path from 'path';

const routes = new Hono();

routes.get('/stream', async (c) => {
  const pathQuery = c.req.query('path');
  const startQuery = c.req.query('start');

  if (!pathQuery || !startQuery) {
    return c.text('Missing path or start query parameter', 400);
  }

  c.header('Content-Type', 'video/mp4');
  c.header('Connection', 'keep-alive');

  const id = `stream-${Date.now()}-${Math.random()}`;
  const child = ffmpegManager.startLiveStream(id, pathQuery, startQuery);

  c.req.raw.signal.addEventListener('abort', () => {
    ffmpegManager.killProcess(id);
  });

  return stream(c, async (streamWriter) => {
    if (!child.stdout) return;

    try {
      for await (const chunk of child.stdout) {
        if (c.req.raw.signal.aborted) {
          ffmpegManager.killProcess(id);
          break;
        }
        await streamWriter.write(chunk);
      }
    } catch (err) {
      console.error('Stream error:', err);
    } finally {
      ffmpegManager.killProcess(id);
    }
  });
});

routes.post('/convert', async (c) => {
  const body = await c.req.json();
  const filePath = body.path;

  if (!filePath) {
    return c.text('Missing path in body', 400);
  }

  const parsedPath = path.parse(filePath);
  const outputPath = path.join(parsedPath.dir, `${parsedPath.name}_wazoo.mkv`);
  const id = `convert-${Date.now()}-${Math.random()}`;

  ffmpegManager.startTranscode(id, filePath, outputPath, (stderrData) => {
    const windows = BrowserWindow.getAllWindows();
    if (windows.length > 0) {
      windows[0].webContents.send('convert-progress', { id, outputPath, stderr: stderrData });
    }
  });

  return c.json({ id, outputPath });
});

export default routes;
