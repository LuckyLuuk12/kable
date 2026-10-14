<script lang="ts">
import { Icon, type GameInstance, type GameInstanceStatus } from "$lib";

let {
  instances,
  selectedInstanceId = $bindable<string | null>(null),
}: {
  instances: GameInstance[];
  selectedInstanceId?: string | null;
} = $props();

let now = $state(Date.now());

$effect(() => {
  const interval = window.setInterval(() => {
    now = Date.now();
  }, 1000);

  return () => {
    window.clearInterval(interval);
  };
});

function getStatusIcon(status: GameInstanceStatus): string {
  switch (status) {
    case "launching":
      return "rocket";
    case "running":
      return "play";
    case "closed":
      return "check";
    case "crashed":
      return "alert";
    case "stopped":
      return "square";
  }
}

function getStatusColor(status: GameInstanceStatus): string {
  switch (status) {
    case "launching":
      return "warning";
    case "running":
      return "success";
    case "closed":
      return "info";
    case "crashed":
      return "danger";
    case "stopped":
      return "secondary";
  }
}

function getInstanceDisplayName(instance: GameInstance): string {
  const duration = Math.max(0, Math.floor((now - instance.launchedAt) / 1000));

  const durationText =
    duration < 60 ? `${duration}s` : duration < 3600 ? `${Math.floor(duration / 60)}m` : `${Math.floor(duration / 3600)}h ${Math.floor((duration % 3600) / 60)}m`;

  return `${instance.profileName} (${durationText})`;
}
</script>

<div class="tabs-container">
  <div class="tab-list">
    <button class:active={selectedInstanceId === null} class="tab-button" onclick={() => (selectedInstanceId = null)}>
      <Icon name="globe" size="sm" />
      <span>Launcher</span>
    </button>

    {#each instances as instance (instance.id)}
      <button class:active={selectedInstanceId === instance.id} class="tab-button" onclick={() => (selectedInstanceId = instance.id)}>
        <Icon name={getStatusIcon(instance.status)} size="sm" />

        <span>{getInstanceDisplayName(instance)}</span>

        <span class="status-badge {getStatusColor(instance.status)}">
          {instance.status}
        </span>
      </button>
    {/each}
  </div>
</div>

<style lang="scss">
.tabs-container {
  .tab-list {
    display: flex;
    gap: 0.05rem;
    background: $color-surface-1;
    border: 1px solid $color-border;
    border-radius: $radius-md $radius-md 0 0;
    overflow-x: auto;

    .tab-button {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.75rem 1rem;
      background: transparent;
      border: none;
      border-right: 1px solid $color-border;
      border-radius: $radius-md $radius-md 0 0;
      color: $color-placeholder;
      font-size: 0.9rem;
      font-weight: 500;
      cursor: pointer;
      white-space: nowrap;
      transition: all 0.2s ease;

      &:last-child {
        border-right: none;
      }

      &:hover {
        color: $color-text;
      }

      &.active {
        background: $color-accent;
        color: $color-text;
      }

      .status-badge {
        padding: 0.125rem 0.375rem;
        border-radius: calc($radius-md * 0.5);
        font-size: 0.85rem;
        font-weight: 600;

        &.success {
          background: color-mix(in srgb, var(--green), 10%, transparent);
          color: var(--green);
        }

        &.warning {
          background: color-mix(in srgb, var(--yellow), 10%, transparent);
          color: var(--yellow);
        }

        &.danger {
          background: color-mix(in srgb, var(--red), 10%, transparent);
          color: var(--red);
        }

        &.info {
          background: color-mix(in srgb, var(--blue), 10%, transparent);
          color: var(--blue);
        }

        &.secondary {
          background: color-mix(in srgb, var(--color-text), 20%, transparent);
          color: var(--color-text);
        }
      }
    }
  }
}
</style>
