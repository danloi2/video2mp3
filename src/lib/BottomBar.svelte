<script>
  import { log, isConverting, totalProgress, queue } from '../stores.js';

  $: done    = $queue.filter(f => f.status === 'done').length;
  $: errors  = $queue.filter(f => f.status === 'error').length;
  $: total   = $queue.length;
  $: pct     = Math.round($totalProgress * 100);
</script>

<footer class="bottom-bar">
  <!-- Overall progress -->
  <div class="progress-section">
    <div class="progress-track">
      <div class="progress-fill" style="width:{pct}%"></div>
    </div>
    <span class="pct-label">
      {#if $isConverting}
        {pct}% Processing...
      {:else if done > 0}
        {done}/{total} files done {#if errors > 0} ({errors} errors){/if}
      {:else}
        Ready
      {/if}
    </span>
  </div>

  <!-- Log -->
  <div class="log-area">
    {#each [...$log].reverse().slice(0, 80) as entry (entry.ts)}
      <div class="log-line" class:ok={entry.ok} class:err={!entry.ok}>
        {entry.msg}
      </div>
    {/each}
  </div>
</footer>

<style>
  .bottom-bar {
    flex-shrink: 0;
    border-top: 1px solid var(--border);
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  /* ── Progress strip ── */
  .progress-section {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 16px 6px;
  }

  .progress-track {
    flex: 1;
    height: 5px;
    background: var(--bg-input);
    border-radius: 99px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), #a78bfa);
    border-radius: 99px;
    transition: width 0.4s ease;
  }

  .pct-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    white-space: nowrap;
    min-width: 80px;
    text-align: right;
  }

  /* ── Log ── */
  .log-area {
    max-height: 90px;
    overflow-y: auto;
    padding: 4px 16px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .log-line {
    font-size: 11.5px;
    font-family: 'Inter', monospace;
    color: var(--text-secondary);
    padding: 1px 0;
    line-height: 1.4;
  }
  .log-line.ok  { color: var(--success); }
  .log-line.err { color: var(--danger); }
</style>
