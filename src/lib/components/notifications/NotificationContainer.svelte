<!-- @component
NotificationContainer - Container for displaying active notifications

Displays all active notifications in a stacked layout in the top-right corner.
Automatically manages notification lifecycle and transitions.
-->
<script lang="ts">
import { notifications } from "$lib/services/NotificationService";
import NotificationToast from "./NotificationToast.svelte";
</script>

<div class="notification-container">
  {#each $notifications as notification (notification.id)}
    <NotificationToast {notification} />
  {/each}
</div>

<style lang="scss">
.notification-container {
  position: fixed;
  top: 1.5rem;
  right: 1.5rem;
  z-index: 10000;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  pointer-events: none;

  > :global(*) {
    pointer-events: auto;
  }
}
</style>
