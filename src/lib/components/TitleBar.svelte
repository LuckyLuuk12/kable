<!-- @component
TitleBar - Custom window title bar with minimize, maximize, and close controls

Provides window controls for the Tauri application window with custom styling.
Displays a DEV badge when running a development build.

@example
```svelte
<TitleBar />
```
-->
<script lang="ts">
import { NotificationContainer, NotificationTray, app, clickSound } from "$lib";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMount, type Snippet } from "svelte";

let { children }: { children?: Snippet } = $props();

let isMaximized = $state(false);
let isDev = $state(false);

const appWindow = getCurrentWindow();

async function minimize() {
  await appWindow.minimize();
}

async function maximize() {
  if (isMaximized) {
    await appWindow.unmaximize();
  } else {
    await appWindow.maximize();
  }

  isMaximized = await appWindow.isMaximized();
}

async function close() {
  await appWindow.close();
}

onMount(() => {
  let unlisten: (() => void) | undefined;

  void (async () => {
    isDev = await app.isDev();
    isMaximized = await appWindow.isMaximized();

    unlisten = await appWindow.onResized(async () => {
      isMaximized = await appWindow.isMaximized();
    });
  })();

  return () => {
    unlisten?.();
  };
});
</script>

<div class="window">
  <div class="titlebar">
    <div class="titlebar-left">
      <img src="/favicon.png" alt="Kable Launcher" class="app-icon" width="24" height="24" />

      <span class="app-title">Kable Launcher</span>

      {#if isDev}
        <button onclick={app.restartAll} class="dev-badge" title="Development build">DEV</button>
      {/if}
    </div>

    <div class="titlebar-right">
      <NotificationTray />

      <button use:clickSound class="titlebar-btn minimize" title="Minimize" aria-label="Minimize" onclick={minimize}>
        <svg width="16" height="16" viewBox="0 0 16 16">
          <rect x="2.5" y="10.5" width="11" height="2" rx="1" fill="currentColor" />
        </svg>
      </button>

      <button
        use:clickSound
        class="titlebar-btn maximize"
        title={isMaximized ? "Restore" : "Maximize"}
        aria-label={isMaximized ? "Restore" : "Maximize"}
        onclick={maximize}>
        {#if isMaximized}
          <svg width="16" height="16" viewBox="0 0 16 16">
            <rect x="4" y="3" width="7" height="7" rx="2" fill="none" stroke="currentColor" stroke-width="2" />
            <rect x="2" y="7" width="7" height="7" rx="2" fill="none" stroke="currentColor" stroke-width="2" />
          </svg>
        {:else}
          <svg width="16" height="16" viewBox="0 0 16 16">
            <rect x="3" y="3" width="10" height="10" rx="2" fill="none" stroke="currentColor" stroke-width="2" />
          </svg>
        {/if}
      </button>

      <button use:clickSound class="titlebar-btn close" title="Close" aria-label="Close" onclick={close}>
        <svg width="16" height="16" viewBox="0 0 16 16">
          <line x1="3" y1="3" x2="13" y2="13" stroke="currentColor" stroke-width="2.5" />
          <line x1="13" y1="3" x2="3" y2="13" stroke="currentColor" stroke-width="2.5" />
        </svg>
      </button>
    </div>
  </div>

  <div class="window-content">
    <NotificationContainer />

    {#if children}
      {@render children()}
    {/if}
  </div>
</div>

<style lang="scss">
.window {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
  min-height: 0;
  overflow: hidden;
}

.titlebar {
  display: flex;
  flex: 0 0 2rem;
  align-items: center;
  justify-content: space-between;
  height: 2rem;
  flex-shrink: 0;
  background: $color-surface-1;
  color: $color-text;
  -webkit-app-region: drag;
  user-select: none;
  padding: 0 0 0 1rem;
  border-bottom: 1px solid $color-border-muted;
}

.window-content {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}
.titlebar-left {
  display: flex;
  align-items: center;
  gap: 1rem;
  min-width: 0;

  .app-icon {
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    border-radius: 6px;
  }

  .app-title {
    font-size: 1.1rem;
    font-weight: 600;
    color: $color-text;
    letter-spacing: 0.02em;
    margin-left: 0.25rem;
    white-space: nowrap;
  }
}

.dev-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.125rem 0.375rem;
  border: 1px solid $color-info;
  border-radius: $radius-sm;
  background: $color-info-muted;
  color: var(--tertiary-500);
  font-size: 0.625rem;
  font-weight: 800;
  letter-spacing: 0.08em;
  line-height: 1.2;
  white-space: nowrap;
  cursor: default;
  -webkit-app-region: no-drag;
}

.titlebar-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.titlebar-btn {
  background: none;
  border: none;
  color: $color-text-muted;
  padding: 0.25rem 0.75rem;
  border-radius: $radius-sm;
  cursor: pointer;
  transition: background 0.2s;
  -webkit-app-region: no-drag;

  &:hover {
    background: $color-surface-2;
    color: $color-text;
  }

  &.close:hover {
    color: $color-error;
    background: transparent;
  }
}
</style>
