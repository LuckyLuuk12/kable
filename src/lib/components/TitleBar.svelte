<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  let isMaximized = false;

  const appWindow = async () => await getCurrentWindow();

  async function minimize() {
    (await appWindow()).minimize();
  }
  async function maximize() {
    if (isMaximized) {
      (await appWindow()).unmaximize();
      isMaximized = false;
    } else {
      (await appWindow()).maximize();
      isMaximized = true;
    }
  }
  async function close() {
    (await appWindow()).close();
  }
  onMount(async () => {
    isMaximized = await (await appWindow()).isMaximized();
    (await appWindow()).onResized(async () => {
      (await appWindow()).isMaximized().then((val: boolean) => isMaximized = val);
    });
  });
</script>

<div class="window">
  <div class="titlebar">
    <div class="titlebar-left">
      <img src="/favicon.png" alt="Kable Launcher" class="app-icon" width="24" height="24" />
      <span class="app-title">Kable Launcher</span>
    </div>
    <div class="titlebar-right">
      <button class="titlebar-btn minimize" title="Minimize" on:click={minimize} aria-label="Minimize">
        <svg width="16" height="16" viewBox="0 0 16 16">
          <rect x="2.5" y="10.5" width="11" height="2" rx="1" fill="currentColor"/>
        </svg>
      </button>
      <button class="titlebar-btn maximize" title={isMaximized ? 'Restore' : 'Maximize'} on:click={maximize}>
        {#if isMaximized}
          <svg width="16" height="16" viewBox="0 0 16 16">
            <!-- Back square, offset up and right, now closer -->
            <rect x="4" y="3" width="7" height="7" rx="2" fill="none" stroke="currentColor" stroke-width="2" />
            <!-- Front square, offset down and left -->
            <rect x="2" y="7" width="7" height="7" rx="2" fill="none" stroke="currentColor" stroke-width="2" />
          </svg>
        {:else}
          <svg width="16" height="16" viewBox="0 0 16 16">
            <rect x="3" y="3" width="10" height="10" rx="2" fill="none" stroke="currentColor" stroke-width="2" />
          </svg>
        {/if}
      </button>
      <button class="titlebar-btn close" title="Close" on:click={close} aria-label="Close">
        <svg width="16" height="16" viewBox="0 0 16 16">
          <line x1="3" y1="3" x2="13" y2="13" stroke="currentColor" stroke-width="2.5"/>
          <line x1="13" y1="3" x2="3" y2="13" stroke="currentColor" stroke-width="2.5"/>
        </svg>
      </button>
    </div>
  </div>
  <slot />
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;
  .window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    max-height: 100vh;
    max-width: 100vw;
  }
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 30px;
    background: $container;
    color: $text;
    -webkit-app-region: drag;
    user-select: none;
    padding: 0 0 0 1rem;
    border-bottom: 1px solid rgba($dark-700, 0.5);
  }
  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 1rem;
    .app-icon {
      width: 24px;
      height: 24px;
      border-radius: 6px;
      box-shadow: 0 2px 8px rgba($primary, 0.08);
    }
    .app-title {
      font-size: 1.1rem;
      font-weight: 600;
      color: $text;
      letter-spacing: 0.02em;
      margin-left: 0.25rem;
    }
  }
  .titlebar-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    -webkit-app-region: no-drag;
  }
  .titlebar-btn {
    background: none;
    border: none;
    color: $placeholder;
    padding: 0.25rem 0.75rem;
    border-radius: $border-radius-small;
    cursor: pointer;
    transition: background 0.2s;
    -webkit-app-region: no-drag;
    &:hover {
      background: $container;
      color: $text;
    }
    &.close:hover {
      background: rgba($red, 0.5);
    }
    &.minimize:hover {
      background: rgba($tertiary-500, 0.05);
    }
    &.maximize:hover {
      background: rgba($secondary-500, 0.05);
    }
  }
</style>
