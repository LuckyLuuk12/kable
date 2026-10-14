<script lang="ts">
import { Icon } from "$lib";
import CacheManager from "./CacheManager.svelte";
import SymlinkManager from "./SymlinkManager.svelte";

let currentTab: "symlinks" | "cache" | "other" = $state("symlinks");
</script>

<div class="page">
  <main class="content">
    <header class="page-header">
      <nav class="navigation" aria-label="Advanced settings">
        <button class="tab-btn" class:active={currentTab === "symlinks"} type="button" onclick={() => (currentTab = "symlinks")}>
          <Icon name="link" size="sm" />
          Symlink Manager
        </button>
        <button class="tab-btn" class:active={currentTab === "cache"} type="button" onclick={() => (currentTab = "cache")}>
          <Icon name="database" forceType="svg" size="sm" />
          Cache Manager
        </button>
        <button class="tab-btn" class:active={currentTab === "other"} type="button" onclick={() => (currentTab = "other")} disabled>
          <Icon name="settings" forceType="svg" size="sm" />
          Other Features
        </button>
      </nav>
    </header>

    <div class="tab-content">
      {#if currentTab === "symlinks"}
        <SymlinkManager />
      {:else if currentTab === "cache"}
        <CacheManager />
      {:else}
        <div class="placeholder">
          <Icon name="settings" size="xl" />
          <h2>Other Features</h2>
          <p>Additional advanced features will be available here.</p>
        </div>
      {/if}
    </div>
  </main>
</div>

<style lang="scss">
.page {
  display: flex;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  color: $color-text;
}

.content {
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-lg;
  flex: 0 0 auto;
  height: $layout-tabbar-height;
  border-bottom: 1px solid $color-border;
}

.navigation {
  display: flex;
  align-items: stretch;
  height: 100%;
}

.tab-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  padding: $space-sm $space-md;
  border: 0;
  border-right: 1px solid transparent;
  border-radius: unset !important;
  background: transparent;
  color: $color-text-muted;
  font: inherit;
  font-size: 0.85rem;
  cursor: pointer;
  transition:
    background 120ms ease,
    color 120ms ease,
    border-color 120ms ease;

  &:hover:not(:disabled) {
    background: $color-hover;
    color: $color-text;
  }

  &.active {
    border-color: $color-border;
    background: $color-surface-2;
    color: $color-text;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: -2px;
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.tab-content {
  flex: 1 1 auto;
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: $space-lg;
}

.placeholder {
  display: flex;
  min-height: 240px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  color: $color-text-muted;
  text-align: center;

  h2,
  p {
    margin: 0;
  }

  h2 {
    color: $color-text;
    font-size: 1.1rem;
  }
}
</style>
