<script>
  import { system, convType, acceleration, preserveGrain, optimizeColor } from '../stores.js';

  const convOptions = [
    { value: 'AudioMP3',  label: '🎵 Audio (MP3)' },
    { value: 'AudioAAC',  label: '🎵 Audio (AAC)' },
    { value: 'VideoMKV',  label: '🎬 Video (MKV — Remux/Copy)' },
    { value: 'VideoH264', label: '🎬 Video (H.264 — Best compatibility)' },
    { value: 'VideoH265', label: '🎬 Video (H.265 — Best compression)' },
  ];

  const accelOptions = [
    { value: 'None',         label: '❌ CPU Only',       hwKey: null },
    { value: 'NVENC',        label: '🚀 NVIDIA (NVENC)', hwKey: 'hw_nvenc' },
    { value: 'QSV',          label: '⚡ Intel (QSV)',    hwKey: 'hw_qsv' },
    { value: 'AMF',          label: '🏎 AMD (AMF)',      hwKey: 'hw_amf' },
    { value: 'VAAPI',        label: '🐧 Linux (VAAPI)',  hwKey: 'hw_vaapi' },
    { value: 'VideoToolbox', label: '🍎 Apple (VTB)',    hwKey: 'hw_vtb' },
  ];

  $: isAudio   = $convType === 'AudioMP3' || $convType === 'AudioAAC';
  $: isRemux   = $convType === 'VideoMKV';
  $: showVideo = !isAudio && !isRemux;
  $: hwTags    = $system ? [
    { label: 'CPU',   color: '#8b91a7', show: true },
    { label: 'NVENC', color: '#76b900', show: $system.hw_nvenc },
    { label: 'QSV',   color: '#0068b5', show: $system.hw_qsv },
    { label: 'AMF',   color: '#ed1c24', show: $system.hw_amf },
    { label: 'VAAPI', color: '#ff8c00', show: $system.hw_vaapi },
    { label: 'APPLE', color: '#a0a0a0', show: $system.hw_vtb },
  ] : [];

  function isAccelAvailable(opt) {
    if (!opt.hwKey || !$system) return opt.value === 'None';
    return $system[opt.hwKey] ?? false;
  }
</script>

<div class="card settings">
  <div class="settings-row">

    <!-- Conversion Mode -->
    <div class="col">
      <label for="conv-type">Conversion Mode</label>
      <select id="conv-type" bind:value={$convType}>
        {#each convOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>

    <div class="divider"></div>

    <!-- Hardware Acceleration -->
    <div class="col">
      <label for="accel">Acceleration
        <span class="hw-tags">
          {#each hwTags.filter(t => t.show) as t}
            <span class="tag" style="background:color-mix(in srgb,{t.color} 15%,transparent);color:{t.color};border:1px solid color-mix(in srgb,{t.color} 35%,transparent)">{t.label}</span>
          {/each}
        </span>
      </label>
      <select id="accel" bind:value={$acceleration} disabled={isAudio || isRemux}>
        {#each accelOptions as opt}
          <option value={opt.value} disabled={!isAccelAvailable(opt)}>
            {opt.label}{isAccelAvailable(opt) ? '' : ' (not detected)'}
          </option>
        {/each}
      </select>
    </div>

    <!-- Video options (only for H.264 / H.265) -->
    {#if showVideo}
      <div class="divider"></div>
      <div class="col">
        <div class="group-label">Options</div>
        <div class="checkboxes">
          <label class="check-label">
            <input type="checkbox" bind:checked={$preserveGrain} />
            🌑 Preserve Grain
          </label>
          <label class="check-label">
            <input type="checkbox" bind:checked={$optimizeColor} />
            🎨 Optimize Color (BT.709)
          </label>
        </div>
      </div>
    {/if}

  </div>
</div>

<style>
  .settings-row {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    flex-wrap: wrap;
  }

  .divider {
    width: 1px;
    background: var(--border);
    align-self: stretch;
    flex-shrink: 0;
  }

  .col { min-width: 180px; }

  label, .group-label {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .hw-tags {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .tag {
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.04em;
  }

  select { width: 100%; min-width: 200px; }

  .checkboxes {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .check-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 400;
    color: var(--text-primary);
    text-transform: none;
    letter-spacing: normal;
    cursor: pointer;
  }

  input[type="checkbox"] {
    width: 15px;
    height: 15px;
    accent-color: var(--accent);
    cursor: pointer;
  }
</style>
