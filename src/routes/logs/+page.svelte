<script lang="ts">
import { app, LogsInstanceTabs, LogsToolbar, LogsViewer } from "$lib";
import type { LogEvent } from "$lib/services/logs.svelte";

let selectedInstanceId = $state<string | null>(null);
let searchTerm = $state("");
let searchMode = $state<"normal" | "regex" | "fuzzy">("fuzzy");
let autoScroll = $state(true);

let logLevelFilters = $state<Record<LogEvent["level"], boolean>>({
  error: true,
  warn: true,
  info: true,
  debug: true,
});

const selectedLogs = $derived.by(() => {
  if (selectedInstanceId === null) {
    return app.logsService.launcherLogs;
  }

  return app.logsService.gameLogs.get(selectedInstanceId) ?? [];
});

const instances = $derived(Array.from(app.logsService.gameInstances.values()));
</script>

<div class="logs-page">
  <LogsToolbar bind:searchTerm bind:searchMode bind:autoScroll bind:logLevelFilters {selectedInstanceId} logs={selectedLogs} />

  <LogsInstanceTabs {instances} bind:selectedInstanceId />

  <LogsViewer logs={selectedLogs} {searchTerm} {searchMode} bind:autoScroll bind:logLevelFilters />
</div>

<style lang="scss">
.logs-page {
  display: flex;
  flex-direction: column;
  min-height: 100%;
  max-height: 100%;
}
</style>
