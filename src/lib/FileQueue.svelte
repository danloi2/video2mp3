<script>
  import {
    queue, isConverting, system,
    convType, acceleration, preserveGrain, optimizeColor, outputDir
  } from '../stores.js';
  import {
    addFiles, addFolder, removeFile, clearQueue,
    selectAll, selectNone, startConversion, cancelConversion
  } from '../app.js';
  import { open } from '@tauri-apps/plugin-dialog';
  import { get } from 'svelte/store';

  $: pending    = $queue.filter(f => f.selected && f.status === 'pending').length;
  $: totalFiles = $queue.length;
  $: selected   = $queue.filter(f => f.selected).length;

  async function convert() {
    const allSelected = get(queue).filter(f => f.selected && f.status === 'pending');
    if (allSelected.length === 0) return;

    const localItems = allSelected.filter(f => !f.youtubeUrl);
    const ytItems    = allSelected.filter(f => f.youtubeUrl);

    // 1. Process local files
    if (localItems.length > 0) {
      await startConversion({
        convType:      get(convType),
        acceleration:  get(acceleration),
        preserveGrain: get(preserveGrain),
        optimizeColor: get(optimizeColor),
        outputDir:     get(outputDir),
      }, localItems);
    }

    // 2. Process YouTube downloads
    if (ytItems.length > 0) {
      await downloadYoutube({
        convType:      get(convType),
        outputDir:     get(outputDir),
      }, ytItems);
    }
  }

  async function changeOutputDir() {
    const dir = await open({ directory: true, title: 'Select output directory' });
    if (dir) outputDir.set(dir);
  }

  function toggleFile(id) {
    queue.update(q => q.map(f => f.id === id ? { ...f, selected: !f.selected } : f));
  }

  function setTrack(id, idx) {
    queue.update(q => q.map(f => f.id === id ? { ...f, selectedTrack: Number(idx) } : f));
  }

  const LANG_MAP = {
    spa:'Español', eng:'English', fra:'Français', deu:'Deutsch', ita:'Italiano',
    jpn:'日本語', zho:'中文', por:'Português', rus:'Русский', kor:'한국어',
    ara:'العربية', nld:'Nederlands', pol:'Polski', swe:'Svenska', tur:'Türkçe',
    cat:'Català', eus:'Euskara', glg:'Galego',
  };
  function langName(code) {
    return LANG_MAP[code?.toLowerCase()] ?? code ?? 'Unknown';
  }

  function statusIcon(s)  { return { pending:'⏳', converting:'⚙️', done:'✅', error:'❌' }[s] ?? '❓'; }
  function statusColor(s) {
    return { pending:'var(--text-muted)', converting:'var(--accent)', done:'var(--success)', error:'var(--danger)' }[s] ?? 'inherit';
  }
</script>

<!-- Toolbar -->
<div class="toolbar">
  <div class="toolbar-left">
    <button class="btn btn-ghost" onclick={addFiles}  disabled={$isConverting}>📂 Add files</button>
    <button class="btn btn-ghost" onclick={addFolder} disabled={$isConverting}>📁 Add folder</button>

    {#if $isConverting}
      <button class="btn btn-danger" onclick={cancelConversion}>⏹ Stop</button>
    {:else}
      <button
        class="btn btn-primary"
        onclick={convert}
        disabled={!$system?.ffmpeg_ok || pending === 0}
      >▶ Convert</button>
    {/if}

    <button class="btn btn-ghost" onclick={clearQueue} disabled={$isConverting}>🗑 Clear</button>

    <div class="sep"></div>

    <button class="btn btn-ghost" onclick={selectAll}  disabled={$isConverting}>✔ All</button>
    <button class="btn btn-ghost" onclick={selectNone} disabled={$isConverting}>✗ None</button>
  </div>

  <div class="toolbar-right">
    <!-- Output directory -->
    <button class="btn btn-ghost small" onclick={changeOutputDir} title="Change output folder">
      📁 {$outputDir ? $outputDir.split('/').pop() : 'Default output'}
    </button>
    {#if $outputDir}
      <button class="btn btn-ghost small" onclick={() => outputDir.set(null)} title="Reset to default">↺</button>
    {/if}

    {#if totalFiles > 0}
      <span class="counter">{selected} / {totalFiles} selected</span>
    {/if}
  </div>
</div>

<!-- File List -->
{#if $queue.length === 0}
  <div class="empty-state">
    <div class="empty-icon">📥</div>
    <p>Drag and drop files here<br/>or use <strong>Add files</strong></p>
  </div>
{:else}
  <div class="file-list">
    {#each $queue as file (file.id)}
      <div class="file-card" class:selected={file.selected} class:is-converting={file.status==='converting'}>

        <!-- Header row -->
        <div class="file-header">
          <input
            type="checkbox"
            checked={file.selected}
            onchange={() => toggleFile(file.id)}
            disabled={$isConverting}
          />
          <span class="status-icon" style="color:{statusColor(file.status)}" title={file.status}>
            {statusIcon(file.status)}
          </span>
          <span class="file-name" title={file.path}>{file.name}</span>

          <div class="file-meta">
            {#if file.container}
              <span class="tag codec">{file.container.toUpperCase()}</span>
            {/if}
            {#if file.vCodec}
              <span class="tag codec video">{file.vCodec.toUpperCase()}</span>
            {/if}
            {#if file.aCodec}
              <span class="tag codec audio">{file.aCodec.toUpperCase()}</span>
            {/if}
          </div>

          {#if !$isConverting}
            <button class="remove-btn" onclick={() => removeFile(file.id)} title="Remove">✕</button>
          {/if}
        </div>

        <!-- Audio track selector -->
        {#if file.tracks.length > 1}
          <div class="track-row">
            <span class="track-label">🎵 Audio track:</span>
            <select
              value={file.selectedTrack}
              onchange={e => setTrack(file.id, e.target.value)}
              disabled={$isConverting}
            >
              {#each file.tracks as t, i}
                <option value={i}>Track {i+1} — {langName(t.language)} ({t.codec.toUpperCase()})</option>
              {/each}
            </select>
          </div>
        {:else if file.tracks.length === 1}
          <div class="track-row">
            <span class="track-label">🎵</span>
            <span class="track-single">Track 1 — {langName(file.tracks[0].language)} ({file.tracks[0].codec.toUpperCase()})</span>
          </div>
        {/if}

        <!-- Progress bar -->
        {#if file.status === 'converting'}
          <div class="progress-bar">
            <div class="progress-fill" style="width:{Math.round(file.progress * 100)}%"></div>
          </div>
        {/if}

        <!-- Result message -->
        {#if file.message && (file.status === 'done' || file.status === 'error')}
          <div class="file-msg" style="color:{statusColor(file.status)}">{file.message}</div>
        {/if}

      </div>
    {/each}
  </div>
{/if}

<!-- System warning -->
{#if $system && !$system.ffmpeg_ok}
  <div class="sys-warn">❌ FFmpeg not found. Install it: <code>sudo apt install ffmpeg</code></div>
{/if}

<style>
  /* ── Toolbar ── */
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;
  }
  .toolbar-left, .toolbar-right { display: flex; align-items: center; gap: 6px; }
  .sep { width: 1px; height: 22px; background: var(--border); flex-shrink: 0; }
  .counter { font-size: 12px; color: var(--text-secondary); white-space: nowrap; }
  .btn.small { padding: 5px 10px; font-size: 12px; }

  /* ── Empty state ── */
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-muted);
    min-height: 200px;
  }
  .empty-icon { font-size: 42px; opacity: 0.4; }
  .empty-state p { font-size: 16px; text-align: center; line-height: 1.7; }
  .empty-state strong { color: var(--text-secondary); }

  /* ── File list ── */
  .file-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow-y: auto;
  }

  .file-card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    transition: border-color 0.15s;
  }
  .file-card.selected    { border-color: color-mix(in srgb, var(--accent) 40%, transparent); }
  .file-card.is-converting { border-color: color-mix(in srgb, var(--accent) 60%, transparent); }

  .file-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  input[type="checkbox"] {
    width: 15px;
    height: 15px;
    accent-color: var(--accent);
    cursor: pointer;
    flex-shrink: 0;
  }

  .status-icon { font-size: 16px; flex-shrink: 0; }

  .file-name {
    flex: 1;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta { display: flex; gap: 4px; flex-shrink: 0; }

  .tag.codec       { background: color-mix(in srgb,#8b91a7 15%,transparent); color:#8b91a7; border:1px solid color-mix(in srgb,#8b91a7 30%,transparent); }
  .tag.codec.video { background: color-mix(in srgb,var(--info) 15%,transparent); color:var(--info); border-color:color-mix(in srgb,var(--info) 30%,transparent); }
  .tag.codec.audio { background: color-mix(in srgb,var(--warning) 15%,transparent); color:var(--warning); border-color:color-mix(in srgb,var(--warning) 30%,transparent); }
  .tag { padding: 1px 6px; border-radius: 4px; font-size: 11px; font-weight: 700; letter-spacing: 0.03em; }

  .remove-btn {
    background: none;
    border: none;
    color: var(--danger);
    cursor: pointer;
    font-size: 13px;
    padding: 2px 6px;
    border-radius: 4px;
    opacity: 0.6;
    transition: opacity 0.15s;
    flex-shrink: 0;
  }
  .remove-btn:hover { opacity: 1; background: color-mix(in srgb,var(--danger) 15%,transparent); }

  /* ── Track selector ── */
  .track-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-left: 26px;
  }
  .track-label { font-size: 13px; color: var(--accent); flex-shrink: 0; }
  .track-single { font-size: 12px; color: var(--text-secondary); }
  .track-row select { font-size: 12px; padding: 4px 8px; }

  /* ── Progress ── */
  .progress-bar {
    height: 3px;
    background: var(--bg-input);
    border-radius: 99px;
    overflow: hidden;
    margin: 2px 0;
  }
  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), #a78bfa);
    border-radius: 99px;
    transition: width 0.3s ease;
  }

  /* ── Messages ── */
  .file-msg { font-size: 12px; padding-left: 26px; }

  /* ── System warning ── */
  .sys-warn {
    background: color-mix(in srgb, var(--danger) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--danger) 30%, transparent);
    border-radius: var(--radius-sm);
    padding: 10px 14px;
    font-size: 13px;
    color: var(--danger);
    margin-top: 4px;
  }
  .sys-warn code { font-family: monospace; background: var(--bg-input); padding: 1px 5px; border-radius: 3px; }
</style>
