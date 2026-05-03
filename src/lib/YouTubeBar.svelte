<script>
  import { system, youtubeUrl, isConverting, convType } from '../stores.js';
  import { addFromYoutube } from '../app.js';

  async function handleAdd() {
    const url = $youtubeUrl.trim();
    if (!url) return;
    await addFromYoutube(url, { convType: $convType });
    youtubeUrl.set('');
  }

  function onKeydown(e) {
    if (e.key === 'Enter') handleAdd();
  }
</script>

<div class="card yt-bar">
  <span class="yt-label">
    <svg width="18" height="18" viewBox="0 0 24 24" fill="#ff0000" aria-hidden="true">
      <path d="M23.5 6.2a3 3 0 0 0-2.1-2.1C19.5 3.6 12 3.6 12 3.6s-7.5 0-9.4.5A3 3 0 0 0 .5 6.2C0 8.1 0 12 0 12s0 3.9.5 5.8a3 3 0 0 0 2.1 2.1c1.9.5 9.4.5 9.4.5s7.5 0 9.4-.5a3 3 0 0 0 2.1-2.1C24 15.9 24 12 24 12s0-3.9-.5-5.8zM9.7 15.5V8.5l6.3 3.5-6.3 3.5z"/>
    </svg>
    YouTube
  </span>
  <input
    type="url"
    placeholder="https://www.youtube.com/watch?v=..."
    bind:value={$youtubeUrl}
    onkeydown={onKeydown}
    class="yt-input"
    disabled={$isConverting}
  />
  <button
    class="btn btn-primary"
    onclick={handleAdd}
    disabled={!$system?.ytdlp_ok || !$youtubeUrl.trim() || $isConverting}
  >
    ➕ Add to list
  </button>
  {#if $system && !$system.ytdlp_ok}
    <span class="warn">⚠ yt-dlp not found</span>
  {/if}
</div>

<style>
  .yt-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    background: color-mix(in srgb, #ff0000 5%, var(--bg-surface));
  }

  .yt-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 700;
    font-size: 14px;
    white-space: nowrap;
    color: var(--text-primary);
  }

  .yt-input {
    flex: 1;
    min-width: 280px;
  }

  .warn {
    color: var(--danger);
    font-size: 12px;
    white-space: nowrap;
  }
</style>
