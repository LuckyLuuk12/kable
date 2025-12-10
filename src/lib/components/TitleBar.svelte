<!-- @component
TitleBar - Custom window title bar with minimize, maximize, and close controls

Provides window controls for the Tauri application window with custom styling.
Handles window state changes and provides standard window management functionality.

@example
```svelte
◄TitleBar /►
```
-->
<script lang="ts">
import { onMount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { NotificationContainer, NotificationTray } from "$lib";
import { clickSound } from "$lib/actions";

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
    (await appWindow())
      .isMaximized()
      .then((val: boolean) => (isMaximized = val));
  });
});
</script>

<div class="window">
  <div class="titlebar">
    <div class="titlebar-left">
      <img
        src="/favicon.png"
        alt="Kable Launcher"
        class="app-icon"
        width="24"
        height="24" />
      <span class="app-title">Kable Launcher</span>
    </div>
    <div class="titlebar-right">
      <!-- Notification Tray -->
      <NotificationTray />

      <button
        use:clickSound
        class="titlebar-btn minimize"
        title="Minimize"
        on:click={minimize}
        aria-label="Minimize">
        <svg width="16" height="16" viewBox="0 0 16 16">
          <rect
            x="2.5"
            y="10.5"
            width="11"
            height="2"
            rx="1"
            fill="currentColor" />
        </svg>
      </button>
      <button
        use:clickSound
        class="titlebar-btn maximize"
        title={isMaximized ? "Restore" : "Maximize"}
        on:click={maximize}>
        {#if isMaximized}
          <svg width="16" height="16" viewBox="0 0 16 16">
            <!-- Back square, offset up and right, now closer -->
            <rect
              x="4"
              y="3"
              width="7"
              height="7"
              rx="2"
              fill="none"
              stroke="currentColor"
              stroke-width="2" />
            <!-- Front square, offset down and left -->
            <rect
              x="2"
              y="7"
              width="7"
              height="7"
              rx="2"
              fill="none"
              stroke="currentColor"
              stroke-width="2" />
          </svg>
        {:else}
          <svg width="16" height="16" viewBox="0 0 16 16">
            <rect
              x="3"
              y="3"
              width="10"
              height="10"
              rx="2"
              fill="none"
              stroke="currentColor"
              stroke-width="2" />
          </svg>
        {/if}
      </button>
      <button
        use:clickSound
        class="titlebar-btn close"
        title="Close"
        on:click={close}
        aria-label="Close">
        <svg width="16" height="16" viewBox="0 0 16 16">
          <line
            x1="3"
            y1="3"
            x2="13"
            y2="13"
            stroke="currentColor"
            stroke-width="2.5" />
          <line
            x1="13"
            y1="3"
            x2="3"
            y2="13"
            stroke="currentColor"
            stroke-width="2.5" />
        </svg>
      </button>
    </div>
  </div>

  <!-- Global Notification Container -->
  <NotificationContainer />

  <slot />
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/variables" as *;
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
  height: 2rem;
  background: var(--container);
  color: var(--text);
  -webkit-app-region: drag;
  user-select: none;
  padding: 0 0 0 1rem;
  border-bottom: 1px solid
    #{"color-mix(in srgb, var(--dark-700), 50%, transparent)"};
}
.titlebar-left {
  display: flex;
  align-items: center;
  gap: 1rem;
  .app-icon {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    box-shadow: 0 2px 8px
      #{"color-mix(in srgb, var(--primary), 8%, transparent)"};
  }
  .app-title {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text);
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
  color: var(--placeholder);
  padding: 0.25rem 0.75rem;
  border-radius: var(--border-radius-small);
  cursor: pointer;
  transition: background 0.2s;
  -webkit-app-region: no-drag;
  &:hover {
    background: var(--container);
    color: var(--text);
  }
  &.close:hover {
    background: #{"color-mix(in srgb, var(--red), 50%, transparent)"};
  }
  &.minimize:hover {
    background: color-mix(in srgb, var(--tertiary-500), 5%, transparent);
  }
  &.maximize:hover {
    background: color-mix(in srgb, var(--secondary-500), 5%, transparent);
  }
}
</style>
