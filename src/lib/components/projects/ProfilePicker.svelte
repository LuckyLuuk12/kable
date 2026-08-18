<!--
@component 
This component binds to a profile variable/state and allows users to select a profile in various ways as specified by optional props.
The default and only/most used one is a kind of 3d-looking vertical carousel of profile cards/badges that the user can scroll through and click in 
to select a profile (similar to how modern "24h clock inputs" work).

Additionally a simple dropdown select and fuzzy search input can be selected instead of the carousel, but these are not the default and are not used in most places.
-->
<script lang="ts">
import { type KableProfile, Icon, app } from "$lib";

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
          aria-selected={item.id === profile?.id}
        >
          <div class="profile-icon">
            <Icon name={loaderIcons[item.id]} size="md" />
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
  flex: 1;
  overflow: hidden;
  padding: 0.5rem;
  position: relative;
  display: flex;
  align-items: center;

  &:focus {
    outline: none;
  }
}

.carousel-container {
  width: 100%;
  height: 100%;
  position: relative;
  display: flex;
  flex-direction: column;
  pointer-events: none;
}

.profile-item {
  padding: 1rem 1.2rem;
  margin: 0;
  border-radius: 0.6rem;
  cursor: pointer;
  border: 1px solid transparent;
  position: absolute;
  top: 35%;
  left: 50%;
  transform-origin: center center;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  width: 100%;
  pointer-events: auto;

  transition: all 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);

  &:hover {
    border-color: var(--loader-color);
    box-shadow: 0 2px 8px color-mix(in srgb, var(--loader-color), 10% transparent);
  }

  &.selected {
    border: 2px solid var(--green-800);

    box-shadow:
      0 4px 16px color-mix(in srgb, var(--loader-color), 15% transparent),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);

    &::before {
      content: "";
      position: absolute;
      left: -0.4rem;
      top: 50%;
      transform: translateY(-50%);
      width: 4px;
      height: 60%;
      background: linear-gradient(to bottom, var(--green-700), var(--green-900));
    }
  }

  &:focus {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--loader-color), 30% transparent);
  }
}

.profile-icon {
  width: calc(48px * var(--carousel-scale, 1));
  height: calc(48px * var(--carousel-scale, 1));
  border-radius: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--container);
  color: var(--loader-color);
  flex-shrink: 0;
  transition: all 0.25s;

  .profile-item.selected & {
    background: linear-gradient(135deg, var(--loader-color) 0%, color-mix(in srgb, var(--loader-color), 80% transparent) 100%);
    color: white;
    transform: scale(1.05);
  }
}

.profile-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.profile-name {
  font-weight: calc(500 + (var(--carousel-scale, 1) * 100));
  margin-bottom: 0.2em;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  transition: all 0.25s;
  font-size: calc(var(--carousel-font-size, 1) * 1em);
  color: var(--text);

  .profile-item.selected & {
    color: var(--loader-color);
    font-weight: 700;
  }
}

.profile-details {
  display: flex;
  gap: 0.4em;
  flex-wrap: wrap;
}

.profile-version {
  font-size: calc(var(--carousel-font-size, 1) * 0.75em);
  padding: 0.15em 0.4em;
  border-radius: 0.3em;
  font-weight: 500;
  transition: all 0.25s;
  opacity: calc(var(--carousel-opacity, 1) * 0.9);
  background: color-mix(in srgb, $color-accent-tertiary, 10% transparent);
  color: $color-accent-tertiary;

  .profile-item.selected & {
    background: color-mix(in srgb, var(--loader-color), 15% transparent);
    color: var(--loader-color);
  }
}
</style>
