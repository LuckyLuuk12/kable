<script lang="ts">
import { app, LogsInstanceTabs, LogsToolbar, LogsViewer } from "$lib";

let selectedInstanceId = $state<string | null>(null);
let searchTerm = $state("");
let searchMode = $state<"normal" | "regex" | "fuzzy">("fuzzy");
let autoScroll = $state(true);

const selectedLogs = $derived(selectedInstanceId ? (app.logsService.gameLogs.get(selectedInstanceId) ?? []) : app.logsService.launcherLogs);

const instances = $derived(Array.from(app.logsService.gameInstances.values()));
</script>

<div class="logs-page">
  <LogsToolbar bind:searchTerm bind:searchMode bind:autoScroll logs={selectedLogs} />

  <LogsInstanceTabs {instances} bind:selectedInstanceId />

  <LogsViewer logs={selectedLogs} {searchTerm} {searchMode} bind:autoScroll />
</div>

<style lang="scss">
.logs-page {
  display: flex;
  flex-direction: column;
  min-height: 100%;
  max-height: 100%;
}
</style>
