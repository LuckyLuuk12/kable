<script lang="ts">
import { ResourcePackBrowser } from '$lib';
</script>

<svelte:head>
  <title>Resource Packs - Kable</title>
</svelte:head>

<div class="page resourcepacks-page">
  <ResourcePackBrowser />
</div>

<style lang="scss">
.resourcepacks-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}
</style>
