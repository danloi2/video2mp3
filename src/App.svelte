<script>
  import { onMount } from 'svelte';
  import TopBar from './lib/TopBar.svelte';
  import Settings from './lib/Settings.svelte';
  import YouTubeBar from './lib/YouTubeBar.svelte';
  import FileQueue from './lib/FileQueue.svelte';
  import BottomBar from './lib/BottomBar.svelte';
  import { initSystem } from './app.js';
  import './styles.css';

  onMount(() => {
    initSystem();
  });
</script>

<div class="app-shell">
  <TopBar />
  <main class="workspace">
    <YouTubeBar />
    <Settings />
    <FileQueue />
  </main>
  <BottomBar />
</div>

<style>
  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }

  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-base);
    color: var(--text-primary);
    font-family: 'Inter', sans-serif;
    overflow: hidden;
  }

  .workspace {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px 16px;
  }
</style>
