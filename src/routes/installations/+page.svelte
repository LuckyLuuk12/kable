<script lang="ts">
import {
  InstallationsList,
  CreateInstallationModal,
  EditInstallationModal,
  Icon,
  type KableInstallation,
  InstallationService,
} from "$lib";
import * as installationsApi from "$lib/api/installations";

let createModalRef: CreateInstallationModal;
let editModalRef: EditInstallationModal;

let isSmall = false;
let isGrid = true;
let isRefreshing = false;
let isRefreshingVersions = false;
let isImporting = false;

// Notification state
let notificationMessage = "";
let notificationType: "success" | "error" | "" = "";
let notificationTimeout: number | null = null;

function showNotification(message: string, type: "success" | "error") {
  notificationMessage = message;
  notificationType = type;

  // Clear any existing timeout
  if (notificationTimeout) {
    clearTimeout(notificationTimeout);
  }

  // Auto-hide after 5 seconds
  notificationTimeout = setTimeout(() => {
    notificationMessage = "";
    notificationType = "";
  }, 5000) as any;
}

function editInstallation(installation: KableInstallation) {
  editModalRef?.open(installation);
}

function openCreateModal() {
  createModalRef?.open();
}

async function refreshInstallations() {
  isRefreshing = true;
  try {
    await InstallationService.refreshInstallations();
  } finally {
    isRefreshing = false;
  }
}

async function refreshVersionManifests() {
  isRefreshingVersions = true;
  try {
    await InstallationService.refreshVersionManifests();
  } finally {
    isRefreshingVersions = false;
  }
}

async function importKableInstallation() {
  try {
    isImporting = true;
    const path = await installationsApi.selectInstallationZip();

    if (path) {
      await InstallationService.importInstallation(path);
      showNotification("Installation imported successfully!", "success");
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : "Unknown error";
    console.error("Failed to import installation:", error);
    showNotification(`Failed to import installation: ${errorMsg}`, "error");
  } finally {
    isImporting = false;
  }
}

async function importFromMinecraftFolder() {
  try {
    isImporting = true;
    const path = await installationsApi.selectMinecraftFolder();

    if (path) {
      const result = await InstallationService.importFromMinecraftFolder(path);
      showNotification(
        "Installations imported successfully from .minecraft folder!",
        "success",
      );
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : "Unknown error";
    console.error("Failed to import from .minecraft folder:", error);
    showNotification(
      `Failed to import from .minecraft folder: ${errorMsg}`,
      "error",
    );
  } finally {
    isImporting = false;
  }
}
</script>

<div class="installations-page">
  <!-- Notification Toast -->
  {#if notificationMessage}
    <div class="notification notification-{notificationType}">
      <Icon
        name={notificationType === "success" ? "check-circle" : "alert"}
        size="sm"
      />
      <span>{notificationMessage}</span>
      <button
        class="close-btn"
        on:click={() => {
          notificationMessage = "";
          notificationType = "";
        }}
      >
        <Icon name="x" size="sm" />
      </button>
    </div>
  {/if}

  <div class="page-header">
    <div class="header-content">
      <h1>Installations</h1>
      <p>Manage your Minecraft installations, versions, and mod loaders</p>
    </div>
  </div>

  <div class="controls-container">
    <div class="left-controls">
      <button
        class="btn btn-primary new-installation-btn"
        on:click={openCreateModal}
      >
        <Icon name="plus" size="md" forceType="svg" />
        New Installation
      </button>
      <button
        class="btn btn-secondary import-btn"
        on:click={importKableInstallation}
        disabled={isImporting}
        title="Import Kable Installation from ZIP file"
      >
        <Icon name="download" size="md" forceType="svg" />
        Import Kable Installation
      </button>
      <button
        class="btn btn-secondary import-btn"
        on:click={importFromMinecraftFolder}
        disabled={isImporting}
        title="Import from existing .minecraft folder"
      >
        <Icon name="folder" size="md" forceType="svg" />
        Import from .minecraft
      </button>
    </div>
    <div class="view-controls">
      <button
        class="btn btn-secondary {isRefreshing ? 'spinning' : ''}"
        on:click={refreshInstallations}
        disabled={isRefreshing}
        title="Refresh installations list"
      >
        <Icon name="refresh" size="md" forceType="svg" />
      </button>
      <button
        class="btn btn-secondary {isRefreshingVersions ? 'spinning' : ''}"
        on:click={refreshVersionManifests}
        disabled={isRefreshingVersions}
        title="Force refresh version manifests from network (useful for new snapshots)"
      >
        <Icon name="sync" size="md" forceType="svg" />
        Refresh Versions
      </button>
      <button
        class="btn btn-secondary"
        on:click={() => (isGrid = !isGrid)}
        class:is-active={isGrid}
        title={isGrid ? "Switch to list view" : "Switch to grid view"}
      >
        <Icon name={isGrid ? "list" : "grid"} size="md" />
      </button>
      <button
        class="btn btn-secondary"
        on:click={() => (isSmall = !isSmall)}
        class:is-active={isSmall}
        title={"Turn compact mode " + (isSmall ? "off" : "on")}
      >
        <Icon name="minimize" size="md" />
      </button>
    </div>
  </div>

  <InstallationsList
    {isGrid}
    {isSmall}
    on:edit={(e) => editInstallation(e.detail)}
  />

  <CreateInstallationModal bind:this={createModalRef} />
  <EditInstallationModal bind:this={editModalRef} />
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/variables" as *;

.installations-page {
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
  .new-installation-btn {
    display: flex;
    align-items: center;
    font-size: 1.1rem;
    padding: 0.75rem 1.5rem;
    background: none;
    color: var(--primary);
    border-radius: var(--border-radius);
    box-shadow: none;
    border: 1.5px solid var(--primary);
    font-weight: 600;
    transition:
      color 0.13s,
      background 0.13s,
      border 0.13s;
    &:hover,
    &:focus {
      background: color-mix(in srgb, var(--primary), 10%, transparent);
      color: var(--primary-900);
      border-color: var(--primary-700);
    }
  }
  .import-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.65rem 1.25rem;
    font-size: 1rem;
    background: var(--card);
    color: var(--text);
    border: 1px solid var(--dark-500);
    border-radius: var(--border-radius);
    font-weight: 500;
    transition:
      background 0.13s,
      color 0.13s,
      border-color 0.13s;
    &:hover:not(:disabled) {
      background: color-mix(in srgb, var(--primary), 10%, transparent);
      color: var(--primary-900);
      border-color: var(--primary-800);
    }
    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
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
        background: color-mix(in srgb, var(--primary), 10%, transparent);
        color: var(--primary-900);
        border-color: var(--primary-800);
      }

      &.is-active {
        background: color-mix(in srgb, var(--primary), 10%, transparent);
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

.notification {
  position: fixed;
  top: 1.5rem;
  right: 1.5rem;
  z-index: 10000;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem 1.25rem;
  border-radius: var(--border-radius);
  box-shadow:
    0 0.5rem 2rem rgba(0, 0, 0, 0.2),
    0 0.25rem 0.5rem rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  font-weight: 500;
  animation: slideIn 0.3s ease-out;
  max-width: 400px;

  &.notification-success {
    background: color-mix(in srgb, var(--green), 15%, var(--card));
    border: 1px solid var(--green);
    color: var(--green);
  }

  &.notification-error {
    background: color-mix(in srgb, var(--red), 15%, var(--card));
    border: 1px solid var(--red);
    color: var(--red);
  }

  span {
    flex: 1;
    font-size: 0.9rem;
  }

  .close-btn {
    background: none;
    border: none;
    padding: 0.25rem;
    cursor: pointer;
    color: inherit;
    opacity: 0.7;
    transition: opacity 0.15s;
    display: flex;
    align-items: center;
    justify-content: center;

    &:hover {
      opacity: 1;
    }
  }
}

@keyframes slideIn {
  from {
    transform: translateX(400px);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
</style>
