import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { queue, log, isConverting, totalProgress, system } from './stores.js';
import { get } from 'svelte/store';

let unlistenProgress = null;
let unlistenFinished = null;
let unlistenYtProgress = null;
let unlistenYtFinished = null;

// ─── System ──────────────────────────────────────────────────────────────────

export async function initSystem() {
  try {
    const status = await invoke('probe_system');
    system.set(status);
  } catch (e) {
    console.error('System probe failed:', e);
  }
}

// ─── File Queue ───────────────────────────────────────────────────────────────

export async function addFiles() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Video', extensions: ['mkv', 'mp4', 'avi', 'mov', 'webm', 'ts', 'flv'] }],
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    for (const path of paths) {
      await addFileToQueue(path);
    }
  } catch (e) {
    appendLog(false, `Error opening files: ${e}`);
  }
}

export async function addFolder() {
  try {
    const selected = await open({ directory: true });
    if (!selected) return;
    appendLog(true, `Folder added: ${selected}`);
    // TODO: list directory files via fs plugin
  } catch (e) {
    appendLog(false, `Error opening folder: ${e}`);
  }
}

async function addFileToQueue(path) {
  const name = path.split('/').pop();
  const id   = `${Date.now()}-${Math.random()}`;

  // Add with minimal info immediately
  queue.update(q => [...q, {
    id, path, name, selected: true, status: 'pending',
    progress: 0, tracks: [], selectedTrack: 0,
  }]);

  // Then enrich with metadata from backend
  try {
    const info = await invoke('scan_file', { path });
    queue.update(q => q.map(f => f.id === id ? {
      ...f,
      container: info.container,
      vCodec:    info.v_codec,
      aCodec:    info.a_codec,
      tracks:    info.tracks,
    } : f));
  } catch (e) {
    console.warn('scan_file failed for', path, e);
  }
}

export function removeFile(id) {
  queue.update(q => q.filter(f => f.id !== id));
}

export function clearQueue() {
  queue.set([]);
  log.set([]);
  totalProgress.set(0);
}

export function selectAll()  { queue.update(q => q.map(f => ({ ...f, selected: true  }))); }
export function selectNone() { queue.update(q => q.map(f => ({ ...f, selected: false }))); }

// ─── Conversion ───────────────────────────────────────────────────────────────

export async function startConversion(settings) {
  const files = get(queue).filter(f => f.selected && f.status === 'pending');
  if (!files.length) return;

  isConverting.set(true);
  totalProgress.set(0);

  // Mark all as converting
  const ids = files.map(f => f.id);
  queue.update(q => q.map(f => ids.includes(f.id) ? { ...f, status: 'converting' } : f));

  // Build job list for backend
  const jobs = files.map((f, idx) => ({
    source:        f.path,
    destination:   null,
    audio_stream:  f.tracks[f.selectedTrack]?.stream_index ?? 0,
    conv_type:     settings.convType,
    acceleration:  settings.acceleration,
    preserve_grain: settings.preserveGrain,
    optimize_color: settings.optimizeColor,
  }));

  // Listen to progress events
  unlistenProgress = await listen('convert:progress', ({ payload }) => {
    const { index, ratio, phase, message } = payload;
    const fileId = ids[index];
    queue.update(q => q.map(f => f.id === fileId ? {
      ...f,
      progress: ratio,
      status:   phase === 'done' ? 'done' : phase === 'error' ? 'error' : 'converting',
      message,
    } : f));

    // Update overall progress
    const q = get(queue);
    const done = q.filter(f => ['done','error'].includes(f.status)).length;
    totalProgress.set(done / ids.length);

    if (message && (phase === 'done' || phase === 'error')) {
      appendLog(phase === 'done', message);
    }
  });

  unlistenFinished = await listen('convert:finished', () => {
    isConverting.set(false);
    totalProgress.set(1);
    cleanup();
    appendLog(true, '✅ Batch complete');
  });

  try {
    await invoke('convert_files', { jobs });
  } catch (e) {
    appendLog(false, `Conversion error: ${e}`);
    isConverting.set(false);
    cleanup();
  }
}

export function cancelConversion() {
  invoke('emit', { event: 'convert:cancel', payload: null }).catch(() => {});
  isConverting.set(false);
  cleanup();
}

// ─── YouTube ──────────────────────────────────────────────────────────────────

export async function addFromYoutube(url, settings) {
  if (!url.trim()) return;
  appendLog(true, `🔍 Scanning: ${url}`);

  try {
    const entries = await invoke('scan_playlist', { url });
    if (!entries.length) {
      appendLog(false, 'No videos found');
      return;
    }
    queue.update(q => [...q, ...entries.map(e => ({
      id:           `yt-${Date.now()}-${Math.random()}`,
      path:         e.url,
      name:         e.title,
      selected:     true,
      status:       'pending',
      progress:     0,
      tracks:       [],
      selectedTrack: 0,
      youtubeUrl:   e.url,
    }))]);
    appendLog(true, `Added ${entries.length} item(s) from YouTube`);
  } catch (e) {
    appendLog(false, `YouTube scan error: ${e}`);
  }
}

export async function downloadYoutube(urls, destination, convType) {
  isConverting.set(true);

  unlistenYtProgress = await listen('yt:progress', ({ payload }) => {
    const { index, ratio, phase, message } = payload;
    // Update corresponding queue item
    const q = get(queue).filter(f => f.youtubeUrl);
    const fileId = q[index]?.id;
    if (fileId) {
      queue.update(all => all.map(f => f.id === fileId ? {
        ...f,
        progress: ratio,
        status:   phase === 'done' ? 'done' : phase === 'error' ? 'error' : 'converting',
        message,
      } : f));
    }
    if (message && (phase === 'done' || phase === 'error')) {
      appendLog(phase === 'done', message);
    }
  });

  unlistenYtFinished = await listen('yt:finished', () => {
    isConverting.set(false);
    cleanup();
    appendLog(true, '✅ YouTube download complete');
  });

  try {
    await invoke('download_youtube_cmd', { urls, destination, convType });
  } catch (e) {
    appendLog(false, `YouTube error: ${e}`);
    isConverting.set(false);
    cleanup();
  }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function appendLog(ok, msg) {
  log.update(l => [...l, { ok, msg, ts: Date.now() }]);
}

function cleanup() {
  unlistenProgress?.();
  unlistenFinished?.();
  unlistenYtProgress?.();
  unlistenYtFinished?.();
  unlistenProgress = unlistenFinished = unlistenYtProgress = unlistenYtFinished = null;
}
