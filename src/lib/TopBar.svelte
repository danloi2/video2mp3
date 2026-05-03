<script>
  import { system } from '../stores.js';
  import pkg from '../../package.json';

  const version = pkg.version;

  function statusColor(ok) { return ok ? 'var(--success)' : 'var(--danger)'; }
  function statusText(ok, ver) { return ok ? ver : 'Missing'; }
</script>

<header class="topbar">
  <div class="brand">
    <span class="logo">🎬</span>
    <div class="brand-text">
      <span class="app-name">video2mp3</span>
      <span class="tagline">Professional Media Converter</span>
    </div>
    <span class="version-badge">v{version}</span>
  </div>

  <div class="status-pills">
    {#if $system}
      <div class="pill" style="--c:{statusColor($system.ffmpeg_ok)}">
        <span class="dot"></span>
        <span class="label">FFmpeg</span>
        <span class="ver">{statusText($system.ffmpeg_ok, $system.ffmpeg_version)}</span>
      </div>
      <div class="pill" style="--c:{statusColor($system.ytdlp_ok)}">
        <span class="dot"></span>
        <span class="label">yt-dlp</span>
        <span class="ver">{statusText($system.ytdlp_ok, $system.ytdlp_version)}</span>
      </div>
    {:else}
      <span class="checking">Checking system…</span>
    {/if}
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 18px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .logo { font-size: 26px; line-height: 1; }

  .brand-text {
    display: flex;
    flex-direction: column;
  }

  .app-name {
    font-size: 17px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .tagline {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 400;
  }

  .version-badge {
    background: var(--accent);
    color: #fff;
    font-size: 11px;
    font-weight: 700;
    padding: 2px 9px;
    border-radius: 99px;
  }

  .status-pills {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .pill {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    border-radius: 99px;
    background: color-mix(in srgb, var(--c) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--c) 30%, transparent);
    font-size: 12px;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--c);
    flex-shrink: 0;
  }

  .label { font-weight: 600; color: var(--text-secondary); }
  .ver   { color: var(--c); font-weight: 500; }

  .checking { color: var(--text-muted); font-size: 12px; animation: pulse 1.5s ease infinite; }
  @keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.4} }
</style>
