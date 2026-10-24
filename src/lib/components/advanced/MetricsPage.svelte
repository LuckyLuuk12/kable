<script lang="ts">
import { error, getJsHeapMetrics, Icon } from "$lib";
import { onMount } from "svelte";

interface MemorySample {
  timestamp: number;
  usedBytes: number;
  totalBytes: number;
  limitBytes: number;
  usedPercent: number;
}

const SAMPLE_INTERVAL_MS = 5_000;
const MAX_SAMPLES = 360;

let samples = $state<MemorySample[]>([]);
let currentMetrics = $state<MemorySample | null>(null);
let monitoring = $state(true);
let copiedReport = $state(false);

let sampleInterval: ReturnType<typeof setInterval> | null = null;
let copyTimeout: ReturnType<typeof setTimeout> | null = null;

const firstSample = $derived(samples[0] ?? null);
const latestSample = $derived(samples.at(-1) ?? null);

const minimumUsedBytes = $derived(samples.length > 0 ? Math.min(...samples.map((sample) => sample.usedBytes)) : 0);

const maximumUsedBytes = $derived(samples.length > 0 ? Math.max(...samples.map((sample) => sample.usedBytes)) : 0);

const averageUsedBytes = $derived(samples.length > 0 ? samples.reduce((total, sample) => total + sample.usedBytes, 0) / samples.length : 0);

const heapGrowthBytes = $derived(firstSample && latestSample ? latestSample.usedBytes - firstSample.usedBytes : 0);

const heapGrowthPercent = $derived(firstSample && firstSample.usedBytes > 0 ? (heapGrowthBytes / firstSample.usedBytes) * 100 : 0);

const monitoringDuration = $derived(firstSample && latestSample ? Math.max(0, latestSample.timestamp - firstSample.timestamp) : 0);

const heapAvailable = $derived(currentMetrics !== null);

const heapStatus = $derived.by(() => {
  if (!currentMetrics) return "unavailable";
  if (currentMetrics.usedPercent >= 90) return "critical";
  if (currentMetrics.usedPercent >= 75) return "warning";
  return "normal";
});

onMount(() => {
  takeSample();
  startMonitoring();

  return () => {
    stopMonitoring();

    if (copyTimeout) {
      clearTimeout(copyTimeout);
      copyTimeout = null;
    }
  };
});

function takeSample(): void {
  const metrics = getJsHeapMetrics();

  if (!metrics) {
    currentMetrics = null;
    return;
  }

  const sample: MemorySample = {
    ...metrics,
    timestamp: Date.now(),
  };

  currentMetrics = sample;

  if (!monitoring && samples.length > 0) {
    return;
  }

  samples = [...samples, sample].slice(-MAX_SAMPLES);
}

function startMonitoring(): void {
  if (sampleInterval !== null) return;

  monitoring = true;
  takeSample();

  sampleInterval = setInterval(() => {
    takeSample();
  }, SAMPLE_INTERVAL_MS);
}

function stopMonitoring(): void {
  monitoring = false;

  if (sampleInterval !== null) {
    clearInterval(sampleInterval);
    sampleInterval = null;
  }
}

function toggleMonitoring(): void {
  if (monitoring) {
    stopMonitoring();
  } else {
    startMonitoring();
  }
}

function resetHistory(): void {
  samples = [];
  takeSample();
}

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  if (bytes < 1024) return `${Math.round(bytes)} B`;

  const units = ["KB", "MB", "GB", "TB"];
  let size = bytes / 1024;
  let unit = 0;

  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit++;
  }

  return `${size.toFixed(2)} ${units[unit]}`;
}

function formatDuration(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000);
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainingSeconds = seconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }

  if (minutes > 0) {
    return `${minutes}m ${remainingSeconds}s`;
  }

  return `${remainingSeconds}s`;
}

function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString();
}

function formatSignedBytes(bytes: number): string {
  if (bytes > 0) return `+${formatBytes(bytes)}`;
  if (bytes < 0) return `−${formatBytes(Math.abs(bytes))}`;
  return formatBytes(0);
}

function formatSignedPercent(percent: number): string {
  if (percent > 0) return `+${percent.toFixed(1)}%`;
  if (percent < 0) return `−${Math.abs(percent).toFixed(1)}%`;
  return "0.0%";
}

function growthClass(value: number): string {
  if (value > 0) return "growth-positive";
  if (value < 0) return "growth-negative";
  return "";
}

function downloadReport(): void {
  const report = {
    generatedAt: new Date().toISOString(),
    samplingIntervalMs: SAMPLE_INTERVAL_MS,
    sampleCount: samples.length,
    currentMetrics,
    samples: samples.map((sample) => ({
      timestamp: new Date(sample.timestamp).toISOString(),
      usedBytes: sample.usedBytes,
      totalBytes: sample.totalBytes,
      limitBytes: sample.limitBytes,
      usedPercent: sample.usedPercent,
    })),
    note: "JavaScript heap measurements are Chromium-specific and do not represent total application process memory or exact live-object memory.",
  };

  const blob = new Blob([JSON.stringify(report, null, 2)], {
    type: "application/json",
  });

  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");

  anchor.href = url;
  anchor.download = `kable-memory-report-${new Date().toISOString().replace(/[:.]/g, "-")}.json`;
  anchor.click();

  URL.revokeObjectURL(url);
}

async function copyCurrentMetrics(): Promise<void> {
  if (!currentMetrics) return;

  const text = [
    "Kable JavaScript Heap",
    `Time: ${new Date(currentMetrics.timestamp).toISOString()}`,
    `Used: ${formatBytes(currentMetrics.usedBytes)}`,
    `Allocated: ${formatBytes(currentMetrics.totalBytes)}`,
    `Limit: ${formatBytes(currentMetrics.limitBytes)}`,
    `Utilization: ${currentMetrics.usedPercent.toFixed(1)}%`,
  ].join("\n");

  try {
    await navigator.clipboard.writeText(text);
    copiedReport = true;

    if (copyTimeout) clearTimeout(copyTimeout);

    copyTimeout = setTimeout(() => {
      copiedReport = false;
      copyTimeout = null;
    }, 2000);
  } catch (err) {
    error("Failed to copy memory metrics:", err);
  }
}

function chartY(value: number): number {
  if (maximumUsedBytes === minimumUsedBytes) return 50;

  const range = maximumUsedBytes - minimumUsedBytes;

  return 92 - ((value - minimumUsedBytes) / range) * 80;
}

function chartX(index: number): number {
  if (samples.length <= 1) return 0;

  return (index / (samples.length - 1)) * 100;
}

const chartPoints = $derived(samples.map((sample, index) => `${chartX(index)},${chartY(sample.usedBytes)}`).join(" "));
</script>

<section class="metrics-page">
  <div class="overview">
    <div class="stat">
      <Icon name="memory" forceType="svg" size="sm" />
      <span class="stat-label">Used heap</span>
      <strong>{currentMetrics ? formatBytes(currentMetrics.usedBytes) : "Unavailable"}</strong>
    </div>

    <span class="stat-divider"></span>

    <div class="stat">
      <Icon name="hamburger" forceType="svg" size="sm" />
      <span class="stat-label">Allocated heap</span>
      <strong>{currentMetrics ? formatBytes(currentMetrics.totalBytes) : "—"}</strong>
    </div>

    <span class="stat-divider"></span>

    <div class="stat">
      <Icon name="settings" forceType="svg" size="sm" />
      <span class="stat-label">Heap limit</span>
      <strong>{currentMetrics ? formatBytes(currentMetrics.limitBytes) : "—"}</strong>
    </div>

    <span class="overview-spacer"></span>

    <div class="header-actions">
      {#if monitoring}
        <span class="monitoring-indicator" role="status">
          <span class="live-dot"></span>
          Monitoring
        </span>
      {:else}
        <span class="paused-indicator">Paused</span>
      {/if}

      <button class="action-btn" type="button" onclick={takeSample} title="Take a sample now" aria-label="Take a sample now">
        <Icon name="refresh" forceType="svg" size="sm" />
      </button>

      <button class="secondary-btn" type="button" onclick={toggleMonitoring}>
        <Icon name={monitoring ? "pause" : "play"} forceType="svg" size="sm" />
        {monitoring ? "Pause" : "Resume"}
      </button>
    </div>
  </div>

  {#if !heapAvailable}
    <div class="notice-panel">
      <Icon name="info" forceType="svg" size="lg" />
      <div>
        <h2>JavaScript heap metrics unavailable</h2>
        <p>
          This WebView does not expose <code>performance.memory</code>. Use Chromium developer tools to inspect the JavaScript heap.
        </p>
      </div>
    </div>
  {:else}
    <div class="metric-grid">
      <article class="metric-card">
        <div class="metric-heading">
          <span>Heap utilization</span>
          <Icon name="wrench" forceType="svg" size="sm" />
        </div>

        <strong class="metric-value">{currentMetrics?.usedPercent.toFixed(1) ?? "—"}%</strong>

        <div
          class="progress-track"
          role="progressbar"
          aria-label="JavaScript heap utilization"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={currentMetrics?.usedPercent ?? 0}>
          <div class={`progress-fill ${heapStatus}`} style={`width: ${Math.min(100, currentMetrics?.usedPercent ?? 0)}%`}></div>
        </div>

        <span class="metric-caption">
          {formatBytes(currentMetrics?.usedBytes ?? 0)} used of {formatBytes(currentMetrics?.limitBytes ?? 0)}
        </span>
      </article>

      <article class="metric-card">
        <div class="metric-heading">
          <span>Heap growth</span>
          <Icon name="arrow-up" forceType="svg" size="sm" />
        </div>

        <strong class={`metric-value ${growthClass(heapGrowthBytes)}`}>
          {formatSignedBytes(heapGrowthBytes)}
        </strong>

        <span class="metric-caption">
          {formatSignedPercent(heapGrowthPercent)} since first sample
        </span>
      </article>

      <article class="metric-card">
        <div class="metric-heading">
          <span>Average used heap</span>
          <Icon name="chart" forceType="svg" size="sm" />
        </div>

        <strong class="metric-value">{formatBytes(averageUsedBytes)}</strong>

        <span class="metric-caption">
          Across {samples.length.toLocaleString()} samples
        </span>
      </article>

      <article class="metric-card">
        <div class="metric-heading">
          <span>Monitoring duration</span>
          <Icon name="clock" forceType="svg" size="sm" />
        </div>

        <strong class="metric-value">{formatDuration(monitoringDuration)}</strong>

        <span class="metric-caption">
          Sampling every {SAMPLE_INTERVAL_MS / 1000} seconds
        </span>
      </article>
    </div>

    <section class="chart-card">
      <header class="section-header">
        <div>
          <h2>Heap history</h2>
          <p>Used JavaScript heap over time. Up to 30 minutes of history.</p>
        </div>

        <span class="sample-count">{samples.length} / {MAX_SAMPLES} samples</span>
      </header>

      {#if samples.length < 2}
        <div class="chart-empty">
          <Icon name="chart-no-axes-combined" forceType="svg" size="xl" />
          <p>Collecting samples…</p>
          <span>At least two samples are required to draw the chart.</span>
        </div>
      {:else}
        <div class="chart-container">
          <div class="chart-labels">
            <span>{formatBytes(maximumUsedBytes)}</span>
            <span>{formatBytes(minimumUsedBytes)}</span>
          </div>

          <svg class="heap-chart" viewBox="0 0 100 100" preserveAspectRatio="none" role="img" aria-label="JavaScript heap usage history">
            <line x1="0" y1="12" x2="100" y2="12" class="chart-gridline" />
            <line x1="0" y1="52" x2="100" y2="52" class="chart-gridline" />
            <line x1="0" y1="92" x2="100" y2="92" class="chart-gridline" />
            <polyline points={chartPoints} class="chart-line" vector-effect="non-scaling-stroke" />
          </svg>
        </div>

        <div class="chart-footer">
          <span>{firstSample ? formatTime(firstSample.timestamp) : "—"}</span>
          <span>Now: {latestSample ? formatTime(latestSample.timestamp) : "—"}</span>
        </div>
      {/if}
    </section>

    <section class="chart-card">
      <header class="section-header">
        <div>
          <h2>Sample statistics</h2>
          <p>Observed range during this monitoring session.</p>
        </div>
      </header>

      <div class="statistics-grid">
        <div class="statistics-item">
          <span>Minimum used heap</span>
          <strong>{formatBytes(minimumUsedBytes)}</strong>
        </div>

        <div class="statistics-item">
          <span>Maximum used heap</span>
          <strong>{formatBytes(maximumUsedBytes)}</strong>
        </div>

        <div class="statistics-item">
          <span>Allocated heap</span>
          <strong>{formatBytes(currentMetrics?.totalBytes ?? 0)}</strong>
        </div>

        <div class="statistics-item">
          <span>Last sample</span>
          <strong>{latestSample ? formatTime(latestSample.timestamp) : "—"}</strong>
        </div>
      </div>
    </section>
  {/if}

  <section class="chart-card">
    <header class="section-header">
      <div>
        <h2>Diagnostics</h2>
        <p>Export measurements for investigating memory growth.</p>
      </div>
    </header>

    <div class="diagnostic-actions">
      <button class="secondary-btn" type="button" onclick={downloadReport} disabled={samples.length === 0}>
        <Icon name="download" forceType="svg" size="sm" />
        Export history
      </button>

      <button class="secondary-btn" type="button" onclick={copyCurrentMetrics} disabled={!currentMetrics}>
        <Icon name={copiedReport ? "check" : "copy"} forceType="svg" size="sm" />
        {copiedReport ? "Copied" : "Copy current metrics"}
      </button>

      <button class="secondary-btn" type="button" onclick={resetHistory}>
        <Icon name="refresh" forceType="svg" size="sm" />
        Reset history
      </button>
    </div>

    <div class="diagnostic-note">
      <Icon name="info" forceType="svg" size="md" />
      <p>
        Heap usage is not total process memory and does not reveal exactly how much memory is awaiting garbage collection. A rising trend alone does not prove a leak. Use
        WebView developer tools and compare heap snapshots after garbage collection to investigate retained objects.
      </p>
    </div>
  </section>
</section>

<style lang="scss">
.metrics-page {
  display: flex;
  flex-direction: column;
  gap: $space-md;
  min-width: 0;
  color: $color-text;
}

.overview {
  display: flex;
  align-items: center;
  gap: $space-lg;
  min-width: 0;
  padding: $space-sm $space-md;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-1;
}

.stat {
  display: flex;
  align-items: center;
  gap: $space-sm;
  min-width: 0;
  color: $color-text-muted;
  font-size: 0.78rem;

  strong {
    color: $color-text;
    font-size: 0.85rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
}

.stat-label {
  white-space: nowrap;
}

.stat-divider {
  width: 1px;
  align-self: stretch;
  background: $color-border-muted;
}

.overview-spacer {
  flex: 1;
}

.header-actions {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: $space-sm;
}

.monitoring-indicator,
.paused-indicator {
  display: inline-flex;
  align-items: center;
  gap: $space-xs;
  color: $color-text-muted;
  font-size: 0.75rem;
  white-space: nowrap;
}

.live-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: $color-success;
  box-shadow: 0 0 8px color-mix(in srgb, $color-success, transparent 40%);
}

.secondary-btn,
.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-text;
  font: inherit;
  cursor: pointer;
  transition:
    background 120ms ease,
    border-color 120ms ease;

  &:hover:not(:disabled) {
    background: $color-hover;
    border-color: $color-border-muted;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.secondary-btn {
  padding: $space-sm $space-md;
  font-size: 0.78rem;
  font-weight: 600;
  white-space: nowrap;
}

.action-btn {
  padding: $space-sm;
  background: transparent;
  color: $color-text-muted;
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: $space-md;
}

.metric-card,
.chart-card {
  min-width: 0;
  padding: $space-md;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-1;
}

.metric-card {
  display: flex;
  flex-direction: column;
  gap: $space-sm;
}

.metric-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-sm;
  color: $color-text-muted;
  font-size: 0.78rem;
}

.metric-value {
  color: $color-text;
  font-size: 1.45rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
  overflow-wrap: anywhere;
}

.metric-caption {
  color: $color-text-muted;
  font-size: 0.73rem;
}

.growth-positive {
  color: $color-warning;
}

.growth-negative {
  color: $color-success;
}

.progress-track {
  height: 6px;
  overflow: hidden;
  border-radius: 50%;
  background: $color-surface-3;
}

.progress-fill {
  height: 100%;
  border-radius: inherit;
  background: $color-accent;
  transition:
    width 250ms ease,
    background 250ms ease;

  &.warning {
    background: $color-warning;
  }

  &.critical {
    background: $color-error;
  }
}

.section-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: $space-md;
  margin-bottom: $space-md;

  h2 {
    margin: 0;
    color: $color-text;
    font-size: 0.9rem;
    font-weight: 600;
  }

  p {
    margin: $space-xs 0 0;
    color: $color-text-muted;
    font-size: 0.75rem;
  }
}

.sample-count {
  flex-shrink: 0;
  color: $color-text-muted;
  font-size: 0.73rem;
  font-variant-numeric: tabular-nums;
}

.chart-container {
  display: flex;
  gap: $space-sm;
  height: 190px;
}

.chart-labels {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 2px 0 12px;
  color: $color-text-muted;
  font-size: 0.68rem;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.heap-chart {
  width: 100%;
  height: 100%;
  overflow: visible;
}

.chart-gridline {
  stroke: $color-border-muted;
  stroke-width: 0.5;
  stroke-dasharray: 2 2;
  vector-effect: non-scaling-stroke;
}

.chart-line {
  fill: none;
  stroke: $color-accent;
  stroke-width: 2;
  stroke-linejoin: round;
  stroke-linecap: round;
}

.chart-footer {
  display: flex;
  justify-content: space-between;
  gap: $space-md;
  margin-top: $space-sm;
  color: $color-text-muted;
  font-size: 0.7rem;
  font-variant-numeric: tabular-nums;
}

.chart-empty {
  display: flex;
  min-height: 160px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  color: $color-text-muted;
  text-align: center;

  p {
    margin: 0;
    color: $color-text;
    font-size: 0.85rem;
  }

  span {
    font-size: 0.75rem;
  }
}

.statistics-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: $space-md;
}

.statistics-item {
  display: flex;
  flex-direction: column;
  gap: $space-xs;
  padding: $space-sm;
  border-radius: $radius-sm;
  background: $color-surface-2;

  span {
    color: $color-text-muted;
    font-size: 0.73rem;
  }

  strong {
    font-size: 0.88rem;
    font-variant-numeric: tabular-nums;
    overflow-wrap: anywhere;
  }
}

.diagnostic-actions {
  display: flex;
  flex-wrap: wrap;
  gap: $space-sm;
}

.diagnostic-note,
.notice-panel {
  display: flex;
  align-items: flex-start;
  gap: $space-sm;
  margin-top: $space-md;
  padding: $space-md;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-text-muted;

  p {
    margin: 0;
    font-size: 0.76rem;
    line-height: 1.55;
  }
}

.notice-panel {
  align-items: center;
  background: $color-surface-1;

  h2 {
    margin: 0 0 $space-xs;
    color: $color-text;
    font-size: 0.9rem;
  }
}

@media (max-width: 900px) {
  .overview {
    flex-wrap: wrap;
    gap: $space-sm $space-md;
  }

  .overview-spacer {
    display: none;
  }

  .header-actions {
    margin-left: auto;
  }

  .metric-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 600px) {
  .overview {
    align-items: flex-start;
  }

  .stat {
    flex-wrap: wrap;
    gap: $space-xs;
  }

  .stat-divider {
    display: none;
  }

  .header-actions {
    width: 100%;
    justify-content: flex-end;
    margin-left: 0;
  }

  .metric-grid,
  .statistics-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .section-header {
    flex-direction: column;
  }

  .chart-container {
    height: 150px;
  }
}
</style>
