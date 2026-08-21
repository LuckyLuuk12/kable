<script lang="ts">
import { app, CreateProfileModal, Icon, ProfilesList } from "$lib";

let isSmall = false;
let isGrid = true;
let isRefreshing = false;
let isRefreshingVersions = false;

function openCreateModal() {
  app.show(CreateProfileModal);
}

async function refreshProfiles() {
  isRefreshing = true;

  try {
    await app.profilesService.refreshProfiles();
  } finally {
    isRefreshing = false;
  }
}

async function refreshVersionManifests() {
  isRefreshingVersions = true;

  try {
    await app.launcherService.refreshVersions();
  } finally {
    isRefreshingVersions = false;
  }
}
</script>

<div class="profiles-page">
  <div class="page-header">
    <div class="header-content">
      <h1>Profiles</h1>
      <p>Manage your Minecraft profiles, versions, and mod loaders</p>
    </div>
  </div>

  <div class="controls-container">
    <div class="left-controls">
      <button
        class="btn new-profiles-btn"
        onclick={openCreateModal}
        title="Create from version, existing, imported or modpack a new profile"
        disabled={isRefreshing || isRefreshingVersions}>
        <Icon name="plus" size="md" forceType="svg" />
        New Profile
      </button>
    </div>
    <div class="view-controls">
      <button class="btn" class:spinning={isRefreshing} onclick={refreshProfiles} disabled={isRefreshing || isRefreshingVersions} title="Refresh profiles list">
        <Icon name="refresh" size="md" forceType="svg" />
        Refresh Profiles
      </button>
      <button
        class="btn"
        class:spinning={isRefreshingVersions}
        onclick={refreshVersionManifests}
        disabled={isRefreshingVersions || isRefreshing}
        title="WARNING: This will clear cache and is very slow, use only if necessary!">
        <Icon name="sync" size="md" forceType="svg" />
        Refresh Versions
      </button>
      <button class="btn" onclick={() => (isGrid = !isGrid)} class:is-active={isGrid} title={isGrid ? "Switch to list view" : "Switch to grid view"}>
        <Icon name={isGrid ? "list" : "grid"} size="md" />
      </button>
      <button class="btn" onclick={() => (isSmall = !isSmall)} class:is-active={isSmall} title={"Turn compact mode " + (isSmall ? "off" : "on")}>
        <Icon name="minimize" size="md" />
      </button>
    </div>
  </div>

  <ProfilesList {isGrid} {isSmall} />
</div>

<style lang="scss">
.profiles-page {
  width: 100%;
  max-width: none;
  margin: 0;
  padding: 0 2vw;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 2rem;

  .header-content {
    text-align: center;
    width: 100%;
    h1 {
      margin: 0 0 0.5rem;
      font-size: 2rem;
      font-weight: 700;
      color: var(--text);
      text-align: center;
    }
    p {
      margin: 0;
      color: var(--placeholder);
      font-size: 1rem;
      text-align: center;
    }
  }
}
.controls-container {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1.5rem;
  flex-wrap: wrap;
  .new-profiles-btn {
    display: flex;
    align-items: center;
    font-size: 1.1rem;
    padding: 0.75rem 1.5rem;
    background: none;
    color: $color-accent;
    border-radius: var(--border-radius);
    box-shadow: none;
    border: 1.5px solid $color-accent;
    font-weight: 600;
    transition:
      color 0.13s,
      background 0.13s,
      border 0.13s;
    &:hover,
    &:focus {
      background: color-mix(in srgb, $color-accent, 10%, transparent);
      color: var(--primary-900);
      border-color: var(--primary-700);
    }
  }
  .left-controls {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }
  .view-controls {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
    button {
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 0.6rem 0.9rem;
      font-size: 1rem;
      border-radius: var(--border-radius);
      background: var(--card);
      color: var(--text);
      border: 1px solid var(--dark-500);
      transition:
        background 0.13s,
        color 0.13s,
        border-color 0.13s;

      &:hover {
        background: color-mix(in srgb, $color-accent, 10%, transparent);
        color: var(--primary-900);
        border-color: var(--primary-800);
      }

      &.is-active {
        background: color-mix(in srgb, $color-accent, 10%, transparent);
        color: var(--primary-900);
        border-color: var(--primary-800);
      }

      &:focus {
        outline: none;
      }
    }
  }
}

:global(.spinning) {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
