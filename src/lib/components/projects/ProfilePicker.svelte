<!--
@component 
This component binds to a profile variable/state and allows users to select a profile in various ways as specified by optional props.
The default and only/most used one is a kind of 3d-looking vertical carousel of profile cards/badges that the user can scroll through and click in 
to select a profile (similar to how modern "24h clock inputs" work).

Additionally a simple dropdown select and fuzzy search input can be selected instead of the carousel, but these are not the default and are not used in most places.
-->
<script lang="ts">
import { type KableProfile, Image, app } from "$lib";

let { profile = $bindable<KableProfile | null>(null) }: { profile?: KableProfile | null } = $props();
let profiles = $derived(app.profilesService.profiles);

let carouselContainer: HTMLElement;
let scrollOffset = 0;
let scrollTimeout: number | undefined;

const sortedProfiles = $derived(
  profiles
    .slice()
    .sort((a, b) => {
      if ((a.metadata.favorite ? 1 : 0) !== (b.metadata.favorite ? 1 : 0)) {
        return (b.metadata.favorite ? 1 : 0) - (a.metadata.favorite ? 1 : 0);
      }

      const aTime = a.metadata.last_used ? new Date(a.metadata.last_used).getTime() : 0;
      const bTime = b.metadata.last_used ? new Date(b.metadata.last_used).getTime() : 0;

      return bTime - aTime;
    })
    .filter((profile) => profile.version.loader !== "vanilla"),
);

const selectedIndex = $derived(sortedProfiles.findIndex((item) => item.id === profile?.id));

const loaderIcons = $derived(Object.fromEntries(sortedProfiles.map((profile) => [profile.id, app.profilesService.getLoaderIcon(profile.version.loader)])));

const loaderColors = $derived(Object.fromEntries(sortedProfiles.map((profile) => [profile.id, app.profilesService.getLoaderColor(profile.version.loader)])));

function selectProfile(nextProfile: KableProfile) {
  profile = nextProfile;
}

function selectRelative(offset: number) {
  if (sortedProfiles.length === 0) {
    return;
  }

  const currentIndex = selectedIndex >= 0 ? selectedIndex : 0;
  const nextIndex = (currentIndex + offset + sortedProfiles.length) % sortedProfiles.length;

  selectProfile(sortedProfiles[nextIndex]);
}

function handleWheel(event: WheelEvent) {
  event.preventDefault();

  if (scrollTimeout) {
    clearTimeout(scrollTimeout);
  }

  scrollOffset += event.deltaY;

  if (Math.abs(scrollOffset) >= 50) {
    selectRelative(scrollOffset > 0 ? 1 : -1);
    scrollOffset = 0;
  }

  scrollTimeout = window.setTimeout(() => {
    scrollOffset = 0;
  }, 200);
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "ArrowDown") {
    event.preventDefault();
    selectRelative(1);
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    selectRelative(-1);
  }
}

function getCarouselScale(currentIndex: number, selectedIndex: number, totalItems: number) {
  if (totalItems === 0) {
    return {
      scale: 0,
      opacity: 0,
      fontSize: 0,
      translateY: 0,
      zIndex: 0,
      visible: false,
    };
  }

  const directDistance = Math.abs(currentIndex - selectedIndex);
  const wrapDistance = totalItems - directDistance;
  const distance = Math.min(directDistance, wrapDistance);

  let relativePosition = currentIndex - selectedIndex;

  if (Math.abs(relativePosition) > totalItems / 2) {
    relativePosition = relativePosition > 0 ? relativePosition - totalItems : relativePosition + totalItems;
  }

  const maxVisibleDistance = Math.min(4, Math.ceil(totalItems / 2));
  const visible = distance <= maxVisibleDistance;

  if (!visible) {
    return {
      scale: 0,
      opacity: 0,
      fontSize: 0,
      translateY: 0,
      zIndex: 0,
      visible: false,
    };
  }

  const containerHeight = carouselContainer?.clientHeight ?? totalItems * 120;
  const baseItemHeight = 120;

  const fitRatio = Math.min(1, containerHeight / Math.max(1, totalItems * baseItemHeight));

  const spacing = 20 * (1 - fitRatio) + 8;

  const baseScaleFactors = [1, 0.85, 0.7, 0.55, 0.4];
  const scaleReduction = 1 - fitRatio * 0.3;

  const scaleFactors = baseScaleFactors.map((scale) => 1 - (1 - scale) * scaleReduction);

  const opacityFactors = [1, 0.85, 0.7, 0.55, 0.4].map((opacity) => opacity * (0.9 + 0.1 * fitRatio));

  const fontFactors = [1, 0.95, 0.9, 0.85, 0.8];

  const index = Math.min(distance, scaleFactors.length - 1);

  const scale = scaleFactors[index];
  const opacity = opacityFactors[index];
  const fontSize = fontFactors[index];

  const distanceNorm = Math.min(distance, 4) / 4;
  const compressionFloor = 0.5;
  const compression = compressionFloor + (1 - compressionFloor) * distanceNorm;

  const translateY = relativePosition * (baseItemHeight * scale + spacing * compression);

  return {
    scale,
    opacity,
    fontSize,
    translateY,
    zIndex: 100 - distance,
    visible: true,
  };
}

$effect(() => {
  if (!profile && sortedProfiles.length > 0) {
    profile = sortedProfiles[0];
  }
});
</script>

<div class="profile-carousel" bind:this={carouselContainer} onwheel={handleWheel} onkeydown={handleKeydown} tabindex="-1" role="listbox">
  <div class="carousel-container">
    {#each sortedProfiles as item, index (item.id)}
      {@const effects = getCarouselScale(index, selectedIndex >= 0 ? selectedIndex : 0, sortedProfiles.length)}

      {#if effects.visible}
        <div
          class="profile-item"
          class:selected={item.id === profile?.id}
          data-profile-id={item.id}
          style="
                        --loader-color: {loaderColors[item.id]};
                        --carousel-scale: {effects.scale};
                        --carousel-opacity: {effects.opacity};
                        --carousel-font-size: {effects.fontSize};
                        background: linear-gradient(
                            135deg,
                            {loaderColors[item.id]}22 0%,
                            {loaderColors[item.id]}08 40%
                        );
                        transform:
                            scale({effects.scale})
                            translateY({effects.translateY}px)
                            translateX(-50%);
                        opacity: {effects.opacity};
                        font-size: calc({effects.fontSize} * 0.9em);
                        z-index: {effects.zIndex};
                    "
          onclick={() => selectProfile(item)}
          onkeydown={(event) => {
            if (event.key === "Enter") {
              selectProfile(item);
            }
          }}
          tabindex="0"
          role="option"
          aria-selected={item.id === profile?.id}>
          <div class="profile-icon">
            <Image key={loaderIcons[item.id]} />
          </div>

          <div class="profile-meta">
            <div class="profile-name">
              {item.metadata.name}
            </div>

            <div class="profile-details">
              <span class="profile-version">
                {item.version.id}
              </span>
            </div>
          </div>
        </div>
      {/if}
    {/each}
  </div>
</div>

<style lang="scss">
.profile-carousel {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  padding: $space-sm;

  &:focus {
    outline: none;
  }
}

.carousel-container {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 0;
  pointer-events: none;
}

.profile-item {
  position: absolute;
  top: 50%;
  left: 50%;

  display: flex;
  align-items: center;
  gap: $space-md;

  width: calc(100% - $space-sm * 2);
  margin: 0;
  padding: $space-md $space-lg;

  border: 1px solid $color-border-muted;
  border-radius: $radius-lg;

  background: $color-surface-2;

  color: $color-text;
  cursor: pointer;

  transform-origin: center center;

  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);

  pointer-events: auto;

  transition:
    transform 0.35s cubic-bezier(0.25, 0.46, 0.45, 0.94),
    opacity 0.35s ease,
    border-color 0.2s ease,
    background-color 0.2s ease,
    box-shadow 0.2s ease;

  &:hover {
    border-color: color-mix(in srgb, var(--loader-color) 45%, $color-border);

    background: $color-surface-3;
  }

  &.selected {
    border-color: color-mix(in srgb, var(--loader-color) 65%, $color-border);

    background: color-mix(in srgb, var(--loader-color) 7%, $color-surface-2);

    box-shadow:
      0 2px 8px rgba(0, 0, 0, 0.12),
      inset 0 0 0 1px color-mix(in srgb, var(--loader-color) 10%, transparent);

    &::before {
      content: "";

      position: absolute;
      left: -1px;
      top: 20%;
      bottom: 20%;

      width: 3px;

      border-radius: $radius-round;

      background: var(--loader-color);
    }
  }

  &:focus-visible {
    outline: 2px solid color-mix(in srgb, var(--loader-color) 60%, transparent);

    outline-offset: 2px;
  }
}

.profile-icon {
  display: flex;
  align-items: center;
  justify-content: center;

  width: calc(44px * var(--carousel-scale, 1));
  height: calc(44px * var(--carousel-scale, 1));

  flex-shrink: 0;

  border: 1px solid color-mix(in srgb, var(--loader-color) 25%, $color-border-muted);

  border-radius: $radius-md;

  background: color-mix(in srgb, var(--loader-color) 10%, $color-surface-1);

  color: var(--loader-color);

  transition:
    width 0.25s ease,
    height 0.25s ease,
    background-color 0.2s ease,
    border-color 0.2s ease;

  .profile-item.selected & {
    border-color: color-mix(in srgb, var(--loader-color) 45%, $color-border-muted);

    background: color-mix(in srgb, var(--loader-color) 18%, $color-surface-1);

    color: var(--loader-color);
  }
}

.profile-meta {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: $space-xs;

  min-width: 0;
}

.profile-name {
  overflow: hidden;

  margin-bottom: 0.15em;

  color: $color-text;

  font-size: calc(var(--carousel-font-size, 1) * 0.9em);
  font-weight: 600;

  text-overflow: ellipsis;
  white-space: nowrap;

  transition:
    color 0.2s ease,
    font-weight 0.2s ease;

  .profile-item.selected & {
    color: var(--loader-color);
    font-weight: 650;
  }
}

.profile-details {
  display: flex;
  gap: 0.4em;
  flex-wrap: wrap;
}

.profile-version {
  display: inline-flex;
  align-items: center;

  padding: $space-1 $space-sm;

  border: 1px solid $color-border-muted;
  border-radius: $radius-round;

  background: $color-surface-1;
  color: $color-text-muted;

  font-size: calc(var(--carousel-font-size, 1) * 0.72em);
  font-weight: 500;
  line-height: 1.2;

  opacity: calc(0.75 + var(--carousel-opacity, 0) * 0.25);

  transition:
    background-color 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease;

  .profile-item.selected & {
    border-color: color-mix(in srgb, var(--loader-color) 35%, $color-border-muted);

    background: color-mix(in srgb, var(--loader-color) 8%, $color-surface-1);

    color: var(--loader-color);
  }
}
</style>
