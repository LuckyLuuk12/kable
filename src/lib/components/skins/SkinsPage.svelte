<script lang="ts">
import { app, Icon } from "$lib";
import { onMount } from "svelte";
import CapeSelection from "./CapeSelection.svelte";
import SkinEditor from "./SkinEditor.svelte";
import SkinSelection from "./SkinSelection.svelte";

type SkinsTab = "skins" | "capes" | "editor";

const skinsService = app.skinsService;

let currentTab = $state<SkinsTab>("skins");
let loading = $state(false);
let error = $state("");

let accountSkins = $derived(skinsService.allSkins);
let capes = $derived(skinsService.capes);
let accountAvailable = $derived(Boolean(app.authService.activeAccount));

onMount(() => {
  void refresh();
});

function errorMessage(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

async function refresh(): Promise<void> {
  if (loading) return;

  loading = true;
  error = "";

  try {
    await skinsService.refresh();

    if (skinsService.error) {
      error = skinsService.error;
    }
  } catch (err) {
    error = `Failed to refresh appearance data: ${errorMessage(err)}`;
  } finally {
    loading = false;
  }
}

function dismissError(): void {
  error = "";
  skinsService.error = null;
}
</script>

<div class="skins-page">
  <header class="page-header">
    <nav class="navigation" aria-label="Appearance">
      <button
        type="button"
        class="tab-btn"
        class:active={currentTab === "skins"}
        aria-current={currentTab === "skins" ? "page" : undefined}
        onclick={() => (currentTab = "skins")}>
        <Icon name="user" forceType="svg" size="sm" />
        <span>Skins</span>
        <span class="tab-count">{accountSkins.length}</span>
      </button>

      <button
        type="button"
        class="tab-btn"
        class:active={currentTab === "capes"}
        aria-current={currentTab === "capes" ? "page" : undefined}
        onclick={() => (currentTab = "capes")}>
        <Icon name="cape" forceType="svg" size="sm" />
        <span>Capes</span>
        <span class="tab-count">{capes.length}</span>
      </button>

      <button
        type="button"
        class="tab-btn"
        class:active={currentTab === "editor"}
        aria-current={currentTab === "editor" ? "page" : undefined}
        onclick={() => (currentTab = "editor")}>
        <Icon name="edit" forceType="svg" size="sm" />
        <span>Editor</span>
      </button>
    </nav>

    <div class="header-actions">
      <span class="account-status">
        {accountAvailable ? (app.authService.activeAccount?.username ?? "Minecraft account") : "No account selected"}
      </span>

      <button type="button" class="icon-button" onclick={refresh} disabled={loading} aria-label="Refresh skins and capes" title="Refresh">
        <Icon name="refresh" forceType="svg" size="sm" />
      </button>

      {#if currentTab === "skins"}
        <button type="button" class="action-button" onclick={() => window.dispatchEvent(new CustomEvent("kable:open-skin-upload"))}>
          <Icon name="upload" forceType="svg" size="sm" />
          Upload skin
        </button>
      {/if}
    </div>
  </header>

  {#if error || skinsService.error}
    <div class="error-message" role="alert">
      <Icon name="alert" forceType="svg" size="sm" />
      <span>{error || skinsService.error}</span>
      <button type="button" class="icon-button" onclick={dismissError} aria-label="Dismiss error">
        <Icon name="close" forceType="svg" size="sm" />
      </button>
    </div>
  {/if}

  <main class="tab-content">
    {#if currentTab === "skins"}
      <SkinSelection />
    {:else if currentTab === "capes"}
      <CapeSelection />
    {:else}
      <SkinEditor />
    {/if}
  </main>
</div>

<style lang="scss">
.skins-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  color: $color-text;
  background: $color-background;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
  flex: 0 0 $layout-tabbar-height;
  min-height: $layout-tabbar-height;
  padding: 0 $space-md;
  border-bottom: 1px solid $color-border;
  background: $color-surface-1;
}

.navigation,
.header-actions {
  display: flex;
  align-items: center;
  min-width: 0;
}

.navigation {
  align-self: stretch;
}

.tab-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  padding: 0 $space-md;
  border: 0;
  border-bottom: 2px solid transparent;
  border-radius: 0;
  background: transparent;
  color: $color-text-muted;
  font: inherit;
  font-size: 0.85rem;
  white-space: nowrap;
  cursor: pointer;

  &:hover {
    background: $color-hover;
    color: $color-text;
  }

  &.active {
    color: $color-text;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: -2px;
  }
}

.tab-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  padding: $space-1 $space-2;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  color: $color-text-muted;
  font-size: 0.7rem;
  font-variant-numeric: tabular-nums;
}

.header-actions {
  justify-content: flex-end;
  gap: $space-sm;
}

.account-status {
  max-width: 220px;
  overflow: hidden;
  color: $color-text-muted;
  font-size: 0.8rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.icon-button,
.action-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  min-height: 32px;
  padding: $space-sm $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: transparent;
  color: $color-text;
  font: inherit;
  font-size: 0.8rem;
  white-space: nowrap;
  cursor: pointer;

  &:hover:not(:disabled) {
    border-color: $color-accent;
    background: $color-hover;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.icon-button {
  width: 32px;
  padding: $space-sm;
  border-color: transparent;
}

.error-message {
  display: flex;
  align-items: center;
  gap: $space-md;
  flex: 0 0 auto;
  margin: $space-md $space-lg 0;
  padding: $space-sm $space-md;
  border: 1px solid $color-error;
  border-radius: $radius-md;
  background: $color-error-muted;
  color: $color-error;
  font-size: 0.85rem;

  > span {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
  }
}

.tab-content {
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

@media (max-width: 720px) {
  .page-header {
    flex-wrap: wrap;
    gap: 0;
    padding: 0 $space-sm;
  }

  .navigation {
    flex: 1 0 100%;
    min-height: $layout-tabbar-height;
  }

  .tab-btn {
    flex: 1;
    padding: 0 $space-sm;
  }

  .header-actions {
    flex: 1 0 100%;
    min-height: $layout-tabbar-height;
    justify-content: flex-end;
    border-top: 1px solid $color-border-muted;
  }

  .account-status {
    margin-right: auto;
    max-width: 45%;
  }
}
</style>
