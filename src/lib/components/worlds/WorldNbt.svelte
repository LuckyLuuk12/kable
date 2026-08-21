<script lang="ts">
import { app, type NbtValue } from "$lib";

let { path }: { path: string } = $props();

let nbt = $state<NbtValue | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

async function load() {
  loading = true;
  error = null;

  try {
    nbt = await app.worldsService.loadNbt(path);

    if (!nbt) {
      error = "Failed to load NBT data.";
    }
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
    nbt = null;
  } finally {
    loading = false;
  }
}

function formatPrimitive(value: NbtValue): string {
  switch (value.type) {
    case "Byte":
    case "Short":
    case "Int":
    case "Long":
    case "Float":
    case "Double":
    case "String":
      return String(value.value);

    case "ByteArray":
    case "IntArray":
    case "LongArray":
      return `[${value.value.join(", ")}]`;

    default:
      return "";
  }
}

function isContainer(value: NbtValue): boolean {
  return value.type === "Compound" || value.type === "List";
}

function children(value: NbtValue): { name: string; value: NbtValue }[] {
  switch (value.type) {
    case "Compound":
      return value.value;

    case "List":
      return value.value.map((child, index) => ({
        name: `[${index}]`,
        value: child,
      }));

    default:
      return [];
  }
}

$effect(() => {
  if (path) {
    load();
  }
});
</script>

<section class="nbt">
  <header class="header">
    <div>
      <h2>NBT Data</h2>
      <p>Raw world NBT structure</p>
    </div>

    <button type="button" onclick={load} disabled={loading}>
      {loading ? "Loading..." : "Reload"}
    </button>
  </header>

  {#if loading}
    <div class="state">Loading NBT data...</div>
  {:else if error}
    <div class="state error">{error}</div>
  {:else if nbt}
    <div class="tree">
      {#each isContainer(nbt) ? children(nbt) : [{ name: "root", value: nbt }] as child (child.name)}
        <div class="node">
          <div class="entry">
            <span class="name">{child.name}</span>
            <span class="type">{child.value.type}</span>

            {#if !isContainer(child.value)}
              <span class="value">{formatPrimitive(child.value)}</span>
            {/if}
          </div>

          {#if isContainer(child.value)}
            <div class="children">
              {#each children(child.value) as nested (nested.name)}
                <div class="node">
                  <div class="entry">
                    <span class="name">{nested.name}</span>
                    <span class="type">{nested.value.type}</span>

                    {#if !isContainer(nested.value)}
                      <span class="value">{formatPrimitive(nested.value)}</span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {:else}
    <div class="state">No NBT data available.</div>
  {/if}
</section>

<style lang="scss">
@use "$lib/styles/variables" as *;

.nbt {
  display: flex;
  flex-direction: column;
  min-height: 0;
  gap: $space-lg;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;

  h2 {
    margin: 0;
    color: $color-text;
    font-size: $font-lg;
    font-weight: $font-weight-semibold;
  }

  p {
    margin: $space-1 0 0;
    color: $color-text-muted;
    font-size: $font-sm;
  }

  button {
    padding: $space-2 $space-md;
    border: 1px solid $color-border;
    border-radius: $radius-md;
    background: $color-surface-2;
    color: $color-text;
    font: inherit;
    cursor: pointer;

    &:hover:not(:disabled) {
      background: $color-hover;
    }

    &:disabled {
      color: $color-disabled;
      cursor: default;
    }
  }
}

.tree {
  min-height: 0;
  overflow: auto;
  padding: $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
  background: $color-surface-1;
  font-family: $font-mono;
  font-size: $font-sm;
}

.node {
  min-width: max-content;
}

.entry {
  display: flex;
  align-items: baseline;
  gap: $space-sm;
  min-height: 1.75rem;
}

.name {
  color: $color-text;
}

.type {
  color: $color-accent-secondary;
  font-size: $font-xs;
}

.value {
  color: $color-text-muted;
  white-space: pre-wrap;
}

.children {
  margin-left: $space-lg;
  padding-left: $space-md;
  border-left: 1px solid $color-border-muted;
}

.state {
  padding: $space-xl;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
  background: $color-surface-1;
  color: $color-text-muted;
  text-align: center;

  &.error {
    color: $color-error;
  }
}
</style>
