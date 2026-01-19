<!-- @component
NotificationToast - Individual notification toast component

Displays a single notification with auto-dismiss, hover-to-persist, and markdown support.

@prop {Notification} notification - The notification to display
-->
<script lang="ts">
import { onMount } from "svelte";
import { Icon, NotificationService } from "$lib";
import type { Notification } from "$lib/services/NotificationService";
import { soundService } from "$lib/services/SoundService";

export let notification: Notification;

onMount(() => {
  // Play notification sound when toast appears
  soundService.playSound("notification");
});

function handleMouseEnter() {
  NotificationService.setHovered(notification.id, true);
}

function handleMouseLeave() {
  NotificationService.setHovered(notification.id, false);
  // Will auto-dismiss after original duration completes
}

function handleDismiss(event: MouseEvent) {
  event.stopPropagation();
  NotificationService.dismiss(notification.id);
}

function handleClick() {
  if (notification.onClick) {
    notification.onClick();
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (notification.onClick && (event.key === "Enter" || event.key === " ")) {
    event.preventDefault();
    notification.onClick();
  }
}

// Map notification types to icons (using existing IconService icons)
const iconMap = {
  success: "check", // or "success"
  error: "error", // or "alert"
  warning: "warning", // or "alert"
  info: "info",
};
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="notification-toast notification-{notification.type}"
  class:clickable={!!notification.onClick}
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  on:click={handleClick}
  on:keydown={handleKeyDown}
  role={notification.onClick ? "button" : "alert"}
  tabindex={notification.onClick ? 0 : -1}
  aria-live="polite"
>
  <div class="notification-icon">
    <Icon name={iconMap[notification.type]} size="sm" forceType="svg" />
  </div>
  <div class="notification-content">
    {#if notification.markdown}
      {@html notification.message}
    {:else}
      <span>{notification.message}</span>
    {/if}
  </div>
  <button
    class="notification-close"
    on:click={handleDismiss}
    aria-label="Dismiss notification"
  >
    <Icon name="x" size="sm" forceType="svg" />
  </button>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/variables" as *;

.notification-toast {
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
  min-width: 300px;
  border: 1px solid transparent;
  transition: transform 0.2s ease;
  cursor: pointer;

  &:hover {
    transform: translateX(-4px);
  }

  &.clickable {
    cursor: pointer;

    &:hover {
      transform: translateX(-6px) scale(1.02);
    }

    &:active {
      transform: translateX(-4px) scale(0.98);
    }
  }

  &.notification-success {
    background: var(--container);
    border-color: var(--green);
    color: var(--green);
  }

  &.notification-error {
    background: var(--container);
    border-color: var(--red);
    color: var(--red);
  }

  &.notification-warning {
    background: var(--container);
    border-color: var(--yellow);
    color: var(--yellow);
  }

  &.notification-info {
    background: var(--container);
    border-color: var(--primary);
    color: var(--primary);
  }
}

.notification-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.notification-content {
  flex: 1;
  font-size: 0.9rem;
  color: inherit;

  // Support for markdown/html content
  :global(a) {
    color: inherit;
    text-decoration: underline;
    font-weight: 600;
  }

  :global(strong) {
    font-weight: 700;
  }

  :global(code) {
    background: rgba(0, 0, 0, 0.2);
    padding: 0.125rem 0.25rem;
    border-radius: 0.25rem;
    font-family: monospace;
    font-size: 0.85em;
  }
}

.notification-close {
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
  flex-shrink: 0;

  &:hover {
    opacity: 1;
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
