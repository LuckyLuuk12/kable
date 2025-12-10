<!-- @component
NotificationTray - Notification history tray component

Displays notification history in a dropdown tray accessible from a bell icon.
Shows recent notifications and allows clearing history.
-->
<script lang="ts">
import { notificationHistory } from "$lib/services/NotificationService";
import { NotificationService, Icon } from "$lib";
import { clickSound } from "$lib/actions";
import { onMount, onDestroy } from "svelte";
import * as systemApi from "$lib/api/system";

let isOpen = false;
let trayElement: HTMLDivElement;

function toggleTray() {
  isOpen = !isOpen;
}

function closeTray() {
  isOpen = false;
}

function clearHistory() {
  NotificationService.clearHistory();
  closeTray();
}

async function openHelp() {
  try {
    await systemApi.openUrl("https://github.com/LuckyLuuk12/kable/wiki");
  } catch (error) {
    console.error("Failed to open help wiki:", error);
    NotificationService.error("Failed to open help page");
  }
}

// Close tray when clicking outside
function handleClickOutside(event: MouseEvent) {
  if (trayElement && !trayElement.contains(event.target as Node)) {
    closeTray();
  }
}

onMount(() => {
  document.addEventListener("click", handleClickOutside);
});

onDestroy(() => {
  document.removeEventListener("click", handleClickOutside);
});

// Map notification types to icons (using existing IconService icons)
const iconMap = {
  success: "check", // or "success"
  error: "error", // or "alert"
  warning: "warning", // or "alert"
  info: "info",
};

// Format timestamp
function formatTime(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (seconds < 60) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  return date.toLocaleDateString();
}
</script>

<div class="notification-tray" bind:this={trayElement}>
  <button
    use:clickSound
    class="tray-toggle"
    on:click|stopPropagation={toggleTray}
    aria-label="Notification history"
    title="Notification history"
  >
    <Icon name="help" size="sm" />
    {#if $notificationHistory.length > 0}
      <span class="notification-badge">{$notificationHistory.length}</span>
    {/if}
  </button>

  {#if isOpen}
    <div class="tray-dropdown">
      <div class="tray-header">
        <h3>Notifications</h3>
        <div class="header-actions">
          <button
            use:clickSound
            class="help-btn"
            on:click={openHelp}
            title="Get help"
          >
            Get Help
          </button>
          {#if $notificationHistory.length > 0}
            <button use:clickSound class="clear-btn" on:click={clearHistory}>
              <Icon name="trash" size="sm" />
              Clear
            </button>
          {/if}
        </div>
      </div>

      <div class="tray-content">
        {#if $notificationHistory.length === 0}
          <div class="empty-state">
            <Icon name="activity" size="lg" />
            <p>No notifications yet</p>
          </div>
        {:else}
          {#each $notificationHistory as notification (notification.id)}
            <div class="tray-item notification-{notification.type}">
              <div class="item-icon">
                <Icon name={iconMap[notification.type]} size="sm" />
              </div>
              <div class="item-content">
                {#if notification.markdown}
                  <div class="item-message">{@html notification.message}</div>
                {:else}
                  <div class="item-message">{notification.message}</div>
                {/if}
                <div class="item-time">
                  {formatTime(notification.timestamp)}
                </div>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/variables" as *;

.notification-tray {
  position: relative;
  display: flex;
  align-items: center;
}

.tray-toggle {
  position: relative;
  background: transparent;
  border: none;
  color: var(--text);
  cursor: pointer;
  padding: 0.5rem;
  border-radius: var(--border-radius);
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;

  &:hover {
    background: var(--button-hover);
    color: var(--primary);
  }
}

.notification-badge {
  position: absolute;
  top: 0.25rem;
  right: 0.25rem;
  background: var(--red);
  color: var(--text-white);
  font-size: 0.65rem;
  font-weight: 700;
  padding: 0.125rem 0.35rem;
  border-radius: 999px;
  min-width: 1rem;
  text-align: center;
  line-height: 1;
}

.tray-dropdown {
  position: absolute;
  top: calc(100% + 0.5rem);
  right: 0;
  width: 360px;
  max-height: 500px;
  background: var(--card);
  border: 1px solid var(--dark-600);
  border-radius: var(--border-radius);
  box-shadow: 0 0.5rem 2rem rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  animation: dropdownSlide 0.2s ease-out;
  z-index: 10001;
}

.tray-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--dark-600);

  h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .help-btn,
  .clear-btn {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    background: transparent;
    border: none;
    font-size: 0.875rem;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: var(--border-radius-small);
    transition: all 0.15s;
  }

  .help-btn {
    color: var(--primary);

    &:hover {
      background: color-mix(in srgb, var(--primary), 10%, transparent);
      color: var(--primary);
    }
  }

  .clear-btn {
    color: var(--placeholder);

    &:hover {
      background: color-mix(in srgb, var(--red), 10%, transparent);
      color: var(--red);
    }
  }
}

.tray-content {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  max-height: 400px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem 1rem;
  color: var(--placeholder);
  gap: 0.5rem;

  p {
    margin: 0;
    font-size: 0.9rem;
  }
}

.tray-item {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 0.75rem;
  border-radius: var(--border-radius);
  margin-bottom: 0.5rem;
  transition: background 0.15s;
  border-left: 3px solid transparent;

  &:hover {
    background: var(--button-hover);
  }

  &.notification-success {
    border-left-color: var(--green);
    .item-icon {
      color: var(--green);
    }
  }

  &.notification-error {
    border-left-color: var(--red);
    .item-icon {
      color: var(--red);
    }
  }

  &.notification-warning {
    border-left-color: var(--yellow);
    .item-icon {
      color: var(--yellow);
    }
  }

  &.notification-info {
    border-left-color: var(--primary);
    .item-icon {
      color: var(--primary);
    }
  }
}

.item-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 0.125rem;
}

.item-content {
  flex: 1;
  min-width: 0;

  .item-message {
    font-size: 0.875rem;
    color: var(--text);
    margin-bottom: 0.25rem;
    word-wrap: break-word;

    :global(code) {
      background: var(--background);
      padding: 0.125rem 0.25rem;
      border-radius: 0.25rem;
      font-family: monospace;
      font-size: 0.85em;
    }
  }

  .item-time {
    font-size: 0.75rem;
    color: var(--placeholder);
  }
}

@keyframes dropdownSlide {
  from {
    opacity: 0;
    transform: translateY(-0.5rem);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
