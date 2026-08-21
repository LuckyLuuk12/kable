<!-- @component
ProfilesList - Displays list or grid of Minecraft profiles

Shows all available profiles with options to launch, edit, duplicate,
delete, export, create shortcuts, and favorite.

@prop {boolean} [isGrid=false] - Display profiles in grid layout
@prop {boolean} [isSmall=false] - Use compact display mode
@prop {string | null} [error=null] - Error message to display
@prop {number | null} [limit=null] - Maximum number of profiles to display
-->
<script lang="ts">
import { app, clickSound, errorSound, Icon, Image, launchSound, type KableProfile } from "$lib";
import EditProfileModal from "./EditProfileModal.svelte";

let {
  isGrid = false,
  isSmall = false,
  error = null,
  limit = null,
} = $props<{
  isGrid?: boolean;
  isSmall?: boolean;
  error?: string | null;
  limit?: number | null;
}>();

let isLoading = $derived(app.profilesService.loading || app.launcherService.loadingVersions);

let profiles = $derived.by(() => {
  const profiles = app.profilesService.profiles;

  if (limit == null) {
    return profiles;
  }

  return profiles.slice(0, limit);
});

function editProfile(profile: KableProfile) {
  app.show(EditProfileModal, {
    profile,
  });
}

async function duplicateProfile(profile: KableProfile) {
  await app.profilesService.createProfile(profile.version.id, profile);
}

async function exportProfile(profile: KableProfile) {
  await app.profilesService.exportProfile(profile);
}

async function createShortcut(profile: KableProfile) {
  try {
    const path = await app.profilesService.createShortcut(profile);
    console.log("Shortcut created at:", path);
  } catch (err) {
    console.error("Failed to create shortcut:", err);
  }
}

async function deleteProfile(profile: KableProfile) {
  await app.profilesService.remove(profile.id);
}

async function toggleFavorite(event: MouseEvent, profile: KableProfile) {
  event.stopPropagation();
  await app.profilesService.toggleFavorite(profile);
}

function getProfileIcon(profile: KableProfile) {
  const icon = profile.metadata.icon;

  if (typeof icon === "string" && (icon.startsWith("data:") || icon.startsWith("http") || icon.startsWith("file:") || icon.startsWith("/"))) {
    return icon;
  }

  return null;
}

function formatPlayedTime(milliseconds: number | null | undefined) {
  if (!milliseconds) {
    return "0h 0m";
  }

  const hours = Math.floor(milliseconds / 3_600_000);
  const minutes = Math.floor((milliseconds % 3_600_000) / 60_000);

  return `${hours}h ${minutes}m`;
}

function formatDate(date: string | null | undefined) {
  return date ? new Date(date).toLocaleDateString() : "Unknown";
}

function formatLastUsed(date: string | null | undefined) {
  return date ? new Date(date).toLocaleDateString() : "Never";
}
</script>

<div class="profiles-list" class:compact={isSmall && !isGrid}>
  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      <span>{error}</span>
    </div>
  {/if}

  {#if isLoading && profiles.length === 0}
    <div class="loading-state">
      <Icon name="refresh" size="md" forceType="svg" />

      <span>Loading profiles...</span>
    </div>
  {:else if profiles.length === 0}
    <div class="empty-state">
      <div class="empty-icon">
        <Icon name="cube" size="xl" />
      </div>

      <h3>No profiles found</h3>
      <p>Create your first Minecraft profile to get started</p>
    </div>
  {:else}
    <div class={isGrid ? "profiles-grid" : "profiles-flex"}>
      {#each profiles as profile, i (profile.id)}
        {@const loaderColor = app.profilesService.getLoaderColor(profile.version.loader)}

        {@const loaderImage = app.profilesService.getLoaderImage(profile.version.loader)}

        {@const customIcon = getProfileIcon(profile)}

        {#if isGrid}
          <div
            class:small={isSmall}
            class="profile-card"
            style="
              background:
                linear-gradient(
                  135deg,
                  {loaderColor}22 0%,
                  {loaderColor}08 40%
                );
              --loader-color: {loaderColor}55;
              z-index: {profiles.length - i};
            ">
            <div class="card-top-actions">
              <button class="star-btn" title={profile.metadata.favorite ? "Unfavorite" : "Favorite"} onclick={(event) => toggleFavorite(event, profile)}>
                {#key profile.metadata.favorite}
                  <Icon name="star" forceType={profile.metadata.favorite ? "emoji" : "svg"} size="md" />
                {/key}
              </button>

              {#if isSmall}
                <div class="small-card-actions">
                  <button use:clickSound class="small-action-btn" onclick={() => editProfile(profile)} title="Edit Profile">
                    <Icon name="edit" size="sm" />
                  </button>

                  <button use:clickSound class="small-action-btn" onclick={() => duplicateProfile(profile)} title="Duplicate Profile">
                    <Icon name="duplicate" size="sm" />
                  </button>

                  <button use:clickSound class="small-action-btn" onclick={() => exportProfile(profile)} title="Export Profile">
                    <Icon name="download" size="sm" />
                  </button>

                  <button use:clickSound class="small-action-btn" onclick={() => createShortcut(profile)} title="Create Shortcut">
                    <Icon name="link" size="sm" />
                  </button>

                  <button use:errorSound class="small-action-btn danger" onclick={() => deleteProfile(profile)} title="Delete Profile">
                    <Icon name="trash" size="sm" />
                  </button>
                </div>
              {/if}
            </div>

            <div class="profile-main">
              <div class="profile-icon-column">
                <div
                  class="profile-icon icon-tooltip-wrapper"
                  style="
                    color: {loaderColor};
                    background: transparent;
                  ">
                  {#if customIcon}
                    <img src={customIcon} alt="profile icon" class="profile-img" />
                  {:else}
                    <Image key={loaderImage ?? "vanilla"} />
                  {/if}

                  <span class="icon-tooltip">
                    {profile.version.loader}
                  </span>
                </div>

                <button
                  use:launchSound
                  class="btn btn-primary play-below-icon"
                  style="
                    background:
                      linear-gradient(
                        90deg,
                        {loaderColor} 60%,
                        {loaderColor}cc 100%
                      );
                    color: $color-text !important;
                  "
                  onclick={async () => {
                    await app.launcherService.launch(profile);
                  }}
                  disabled={app.launcherService.launching}>
                  {#if app.launcherService.launchingProfileId === profile.id}
                    <Icon name="refresh" size="sm" className="spin" forceType="svg" />

                    <span>Launching...</span>
                  {:else}
                    Play
                  {/if}
                </button>
              </div>

              <div class="profile-meta">
                <div class="profile-title-row">
                  <h3>
                    {profile.metadata.name || profile.version.id}
                  </h3>
                </div>

                {#if profile.version.id}
                  <div class="loader-version-row">
                    <span class="loader-version" style="color: {loaderColor};">
                      {profile.version.id}
                    </span>
                  </div>
                {/if}

                {#if isSmall}
                  <div class="profile-meta-grid small-meta-grid">
                    <div class="meta-cell small-meta-cell">
                      <span class="meta-key"> Total time: </span>

                      <span class="meta-value last-played small-meta-value">
                        <Icon name="clock" size="sm" />

                        {formatPlayedTime(profile.metadata.total_time_played_ms)}
                      </span>
                    </div>
                  </div>
                {:else}
                  <div class="profile-meta-grid">
                    <div class="meta-cell">
                      <span class="meta-key">Created:</span>

                      <span class="meta-value">
                        <Icon name="calendar" size="sm" />

                        {formatDate(profile.metadata.created)}
                      </span>
                    </div>

                    <div class="meta-cell">
                      <span class="meta-key">Last played:</span>

                      <span class="meta-value">
                        <Icon name="clock" size="sm" />

                        {formatLastUsed(profile.metadata.last_used)}
                      </span>
                    </div>

                    <div class="meta-cell">
                      <span class="meta-key">Total time:</span>

                      <span class="meta-value">
                        <Icon name="clock" size="sm" />

                        {formatPlayedTime(profile.metadata.total_time_played_ms)}
                      </span>
                    </div>
                  </div>
                {/if}
              </div>
            </div>

            {#if !isSmall}
              <div class="profile-actions">
                <button use:clickSound class="btn btn-secondary" onclick={() => editProfile(profile)} title="Edit Profile">
                  <Icon name="edit" size="sm" />
                  Edit
                </button>

                <button use:clickSound class="btn btn-secondary" onclick={() => duplicateProfile(profile)} title="Duplicate Profile">
                  <Icon name="duplicate" size="sm" />
                  Duplicate
                </button>

                <button use:clickSound class="btn btn-secondary" onclick={() => exportProfile(profile)} title="Export Profile">
                  <Icon name="download" size="sm" />
                  Export
                </button>

                <button use:clickSound class="btn btn-secondary" onclick={() => createShortcut(profile)} title="Create Shortcut">
                  <Icon name="link" size="sm" />
                  Shortcut
                </button>

                <button use:errorSound class="btn btn-danger" onclick={() => deleteProfile(profile)} title="Delete Profile">
                  <Icon name="trash" size="sm" />
                  Delete
                </button>
              </div>
            {/if}
          </div>
        {:else}
          <div
            class="profile-list-item"
            style="
              background:
                linear-gradient(
                  135deg,
                  {loaderColor}15 0%,
                  {loaderColor}05 40%
                );
              --loader-color: {loaderColor}55;
            ">
            <div class="list-item-main">
              <div class="list-item-icon-section">
                <div class="profile-icon icon-tooltip-wrapper" style="color: {loaderColor};">
                  {#if customIcon}
                    <img src={customIcon} alt="profile icon" class="profile-img list-img" />
                  {:else}
                    <Image key={loaderImage ?? "vanilla"} />
                  {/if}

                  <span class="icon-tooltip">
                    {profile.version.loader}
                  </span>
                </div>

                <button
                  use:launchSound
                  class="btn btn-primary list-play-btn"
                  style="
                    background:
                      linear-gradient(
                        90deg,
                        {loaderColor} 60%,
                        {loaderColor}cc 100%
                      );
                    color: $color-text !important;
                  "
                  onclick={async () => {
                    await app.launcherService.launch(profile);
                  }}
                  disabled={app.launcherService.launching}>
                  {#if app.launcherService.launchingProfileId === profile.id}
                    <Icon name="refresh" size="sm" className="spin" forceType="svg" />

                    <span>Launching...</span>
                  {:else}
                    Play
                  {/if}
                </button>
              </div>

              <div class="list-item-content">
                <div class="list-title-actions-row">
                  <div class="list-title">
                    <h3>
                      {profile.metadata.name || profile.version.id}
                    </h3>

                    {#if profile.version.id && profile.metadata.name}
                      <span class="list-version" style="color: {loaderColor};">
                        {profile.version.id}
                      </span>
                    {/if}
                  </div>

                  <div class="list-actions-section">
                    <button class="star-btn" title={profile.metadata.favorite ? "Unfavorite" : "Favorite"} onclick={(event) => toggleFavorite(event, profile)}>
                      {#key profile.metadata.favorite}
                        <Icon name="star" forceType={profile.metadata.favorite ? "emoji" : "svg"} size="sm" />
                      {/key}
                    </button>

                    <div class="list-inline-actions">
                      <button use:clickSound class="list-action-btn" onclick={() => editProfile(profile)} title="Edit Profile">
                        <Icon name="edit" size="sm" />
                        <span>Edit</span>
                      </button>

                      <button use:clickSound class="list-action-btn" onclick={() => duplicateProfile(profile)} title="Duplicate Profile">
                        <Icon name="duplicate" size="sm" />
                        <span>Duplicate</span>
                      </button>

                      <button use:clickSound class="list-action-btn" onclick={() => exportProfile(profile)} title="Export Profile">
                        <Icon name="download" size="sm" />
                        <span>Export</span>
                      </button>

                      <button use:clickSound class="list-action-btn" onclick={() => createShortcut(profile)} title="Create Shortcut">
                        <Icon name="link" size="sm" />
                        <span>Shortcut</span>
                      </button>

                      <button use:errorSound class="list-action-btn danger" onclick={() => deleteProfile(profile)} title="Delete Profile">
                        <Icon name="trash" size="sm" />
                        <span>Delete</span>
                      </button>
                    </div>
                  </div>
                </div>

                <div class="list-version-stats-row">
                  <div class="list-stats-section">
                    <div class="list-meta-item">
                      <Icon name="calendar" size="sm" />

                      <span>
                        {formatDate(profile.metadata.created)}
                      </span>
                    </div>

                    <div class="list-meta-item">
                      <Icon name="clock" size="sm" />

                      <span>
                        {formatLastUsed(profile.metadata.last_used)}
                      </span>
                    </div>

                    <div class="list-meta-item">
                      <Icon name="clock" size="sm" />

                      <span>
                        {formatPlayedTime(profile.metadata.total_time_played_ms)}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
.profiles-list {
  min-height: 100%;
  padding: 2rem;
  border-radius: $radius-md;
  border: 1px solid $color-surface-3;
  background: $color-surface-1;
  box-shadow: 0 0.125rem 0.25rem rgba(0, 0, 0, 0.08);
  overflow: visible;
}

.profiles-list.compact {
  padding: 1rem;

  .profile-list-item {
    margin-bottom: 0.5rem;

    .list-item-main {
      padding: 0.75rem;
      gap: 0.75rem;
    }

    .list-item-content {
      h3 {
        font-size: 0.9rem;
        margin-bottom: 0.25rem;
      }

      .list-version-stats-row {
        font-size: 0.8rem;
      }

      .list-meta-item {
        font-size: 0.8rem;
      }
    }
  }
}

.profiles-grid {
  min-height: 100%;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(20.5rem, 1fr));
  gap: 1.25rem;
  align-items: stretch;
}

.profiles-flex {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.profile-img {
  display: block;
  width: 48px;
  height: 48px;
  max-width: 48px;
  max-height: 48px;
  object-fit: contain;
  border-radius: $radius-md;
}

.profile-img.list-img {
  width: 32px;
  height: 32px;
  max-width: 32px;
  max-height: 32px;
}

.profile-icon {
  position: relative;

  display: flex;
  align-items: center;
  justify-content: center;

  flex-shrink: 0;

  width: 2.5rem;
  height: 2.5rem;

  border-radius: $radius-md;
  background: $color-surface-1;

  font-size: 1.5rem;
  cursor: pointer;
}

/* ============================================================
   GRID CARD
   ============================================================ */

.profile-card {
  position: relative;

  min-width: 0;
  min-height: 0;

  display: grid;
  grid-template-rows: auto auto 1fr auto;

  padding: 1rem;
  border-radius: $radius-md;
  border: 1px solid transparent;

  background: $color-surface-2;

  box-shadow:
    0 0.125rem 0.75rem rgba(0, 0, 0, 0.07),
    0 0.09375rem 0.25rem rgba(0, 0, 0, 0.04);

  backdrop-filter: blur(0.5rem);
  -webkit-backdrop-filter: blur(0.5rem);

  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease,
    transform 0.15s ease;

  &:hover {
    border-color: $color-surface-3;

    box-shadow:
      0 0.375rem 1.5rem rgba(0, 0, 0, 0.13),
      0 0.125rem 0.5rem rgba(0, 0, 0, 0.07);
  }

  &.small {
    padding: 0.8rem;
  }

  > .card-top-actions {
    position: absolute;
    top: 0.55rem;
    right: 0.55rem;
    z-index: 10;

    display: flex;
    align-items: center;

    pointer-events: none;

    .star-btn {
      pointer-events: auto;
    }

    .small-card-actions {
      display: none !important;
    }
  }

  .star-btn {
    display: flex;
    align-items: center;
    justify-content: center;

    width: 2rem;
    height: 2rem;
    padding: 0;

    background: transparent;
    border: none;
    border-radius: 50%;

    box-shadow: none;
    cursor: pointer;

    transition:
      transform 0.15s ease,
      background 0.15s ease;

    &:hover,
    &:focus-visible {
      background: $color-surface-3;
      transform: scale(1.1);
      outline: none;
    }
  }

  .profile-dropdown,
  .actions-dropdown,
  .dropdown {
    display: none !important;
  }
}

/* ============================================================
   GRID CARD CONTENT
   ============================================================ */

.profile-main {
  display: grid;
  border-radius: $radius-lg;
  grid-template-columns: 5rem minmax(0, 1fr);
  align-items: start;

  min-width: 0;
  gap: 0.85rem;

  margin: 0;
  padding: 0.05rem 0 0;
}

.profile-icon-column {
  min-width: 0;

  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.55rem;
}

.profile-icon-column > .profile-icon {
  width: 2.75rem;
  height: 2.75rem;
}

.play-below-icon {
  width: 100%;
  min-width: 0;
  max-width: 5rem;

  padding: 0.45rem 0.55rem;

  align-self: center;

  border: none;
  border-radius: $radius-md;

  font-weight: 700;
  letter-spacing: 0.02em;
  color: $color-text !important;

  box-shadow: 0 0.125rem 0.75rem rgba(0, 0, 0, 0.1);

  transition:
    filter 0.15s ease,
    box-shadow 0.15s ease,
    transform 0.15s ease;

  &:hover,
  &:focus-visible {
    filter: brightness(1.1) saturate(1.1);
    box-shadow: 0 0.375rem 1rem rgba(0, 0, 0, 0.18);
    transform: translateY(-1px);
    outline: none;
  }
}

.profile-meta {
  min-width: 0;

  display: flex;
  flex-direction: column;
  gap: 0.3rem;

  overflow: hidden;
}

.profile-title-row {
  min-width: 0;

  display: flex;
  align-items: center;

  h3 {
    min-width: 0;
    max-width: 100%;
    margin: 0;

    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;

    color: $color-text;
    font-size: 1.1rem;
    font-weight: 800;
    line-height: 1.25;
  }
}

.loader-version-row {
  min-width: 0;
  width: 100%;

  display: flex;
  align-items: center;

  margin: 0.05rem 0 0.15rem;
}

.loader-version {
  display: inline-block;

  min-width: 0;
  max-width: 100%;

  padding: 0.08em 0.4em;

  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  border-radius: $radius-md;
  background: $color-surface-1;

  font-size: 0.78rem;
  font-weight: 500;
  line-height: 1.2;
}

.profile-meta-grid {
  min-width: 0;

  display: flex;
  flex-wrap: wrap;
  align-items: center;

  gap: 0.3rem 0.6rem;

  margin-top: 0.25rem;

  font-size: 0.65rem;

  .meta-cell {
    min-width: 0;

    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
}

.meta-key {
  min-width: fit-content;

  color: $color-text;
  font-weight: 500;
  opacity: 0.8;
  white-space: nowrap;
}

.meta-value {
  min-width: 0;

  display: flex;
  align-items: center;
  gap: 0.25em;

  padding: 0.05em 0.4em;

  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  border-radius: $radius-md;
  background: $color-surface-3;

  color: $color-text-muted;
  font-weight: 400;
}

.small-meta-grid {
  display: flex !important;
  flex-direction: row !important;

  font-size: 0.82rem !important;
}

.small-meta-cell {
  width: auto;

  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.small-meta-value {
  flex: 0 1 auto;
  min-width: 0;
  max-width: 100%;

  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  font-size: 0.82rem;
}

/* ============================================================
   GRID ACTIONS
   ============================================================ */

.profile-actions {
  min-width: 0;

  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  align-items: stretch;

  gap: 0.1rem;

  margin-top: 0.55rem;
  padding-top: 0.55rem;

  border-top: 1px solid $color-surface-3;

  button {
    min-width: 0;
    width: 100%;

    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;

    background: transparent !important;
    border: none;
    border-radius: $radius-md;
    box-shadow: none;

    color: $color-text-muted;
    font-size: 0.7rem;
    font-weight: 600;

    white-space: nowrap;
    overflow: hidden;

    cursor: pointer;

    transition:
      color 0.13s ease,
      background 0.13s ease;

    :global(.icon) {
      flex-shrink: 0;
    }

    &:hover,
    &:focus-visible {
      color: $color-text;
      background: $color-surface-3 !important;
      outline: none;
    }

    &.btn-danger,
    &.btn.btn-danger {
      color: $color-error;

      &:hover,
      &:focus-visible {
        color: $color-error;
        background: color-mix(in srgb, $color-error, transparent 90%) !important;
      }
    }
  }
}

.profile-card.small {
  .profile-actions {
    display: none !important;
  }
}

/* ============================================================
   LIST CARD
   ============================================================ */

.profile-list-item {
  position: relative;
  z-index: 1;

  min-width: 0;

  background: $color-surface-2;
  border: 1px solid transparent;
  border-radius: $radius-md;

  box-shadow:
    0 0.125rem 0.5rem rgba(0, 0, 0, 0.06),
    0 0.0625rem 0.125rem rgba(0, 0, 0, 0.04);

  backdrop-filter: blur(0.5rem);
  -webkit-backdrop-filter: blur(0.5rem);

  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;

  &:hover {
    border-color: $color-surface-3;

    box-shadow:
      0 0.25rem 1rem rgba(0, 0, 0, 0.1),
      0 0.125rem 0.25rem rgba(0, 0, 0, 0.06);
  }

  .profile-dropdown,
  .actions-dropdown,
  .dropdown,
  .dropdown-toggle,
  .dropdown-menu,
  .dropdown-content,
  .dropdown-options,
  .dropdown-separator {
    display: none !important;
    visibility: hidden !important;
    pointer-events: none !important;
  }
}

.list-item-main {
  min-width: 0;

  display: flex;
  align-items: center;

  gap: 1rem;

  padding: 0.9rem 1.1rem;
}

.list-item-icon-section {
  flex: 0 0 auto;

  display: flex;
  align-items: center;
  gap: 0.75rem;

  .profile-icon {
    width: 2.5rem;
    height: 2.5rem;

    border-radius: $radius-md;
    background: $color-surface-1;

    font-size: 1.25rem;
  }
}

.list-play-btn {
  min-width: 4.25rem;

  padding: 0.5rem 0.8rem;

  border: none;
  border-radius: $radius-md;

  font-size: 0.875rem;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: $color-text !important;

  box-shadow: 0 0.125rem 0.5rem rgba(0, 0, 0, 0.08);

  transition:
    filter 0.15s ease,
    box-shadow 0.15s ease,
    transform 0.15s ease;

  &:hover,
  &:focus-visible {
    filter: brightness(1.08) saturate(1.1);
    box-shadow: 0 0.25rem 1rem rgba(0, 0, 0, 0.15);
    transform: translateY(-1px);
    outline: none;
  }
}

.list-item-content {
  flex: 1 1 0%;
  min-width: 0;

  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.list-title-actions-row {
  min-width: 0;

  display: flex;
  align-items: center;

  gap: 1.5rem;
}

.list-title {
  min-width: 0;
  flex: 1 1 auto;

  display: flex;
  align-items: center;
  gap: 0.75rem;

  h3 {
    min-width: 0;
    flex: 0 1 auto;

    margin: 0;

    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;

    color: $color-text;
    font-size: 1rem;
    font-weight: 700;
  }
}

.list-version {
  flex: 0 1 auto;
  min-width: 0;
  max-width: 18rem;

  padding: 0.15rem 0.55rem;

  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  border-radius: $radius-md;
  background: $color-surface-1;

  font-size: 0.75rem;
  font-weight: 500;
}

.list-actions-section {
  flex: 0 0 auto;
  min-width: 0;

  display: flex;
  align-items: center;
  justify-content: flex-end;

  gap: 0.5rem;

  .star-btn {
    position: static;

    flex: 0 0 auto;

    width: 2rem;
    height: 2rem;
    padding: 0;

    display: flex;
    align-items: center;
    justify-content: center;

    background: transparent;
    border: none;
    border-radius: 50%;

    cursor: pointer;

    transition:
      transform 0.15s ease,
      background 0.15s ease;

    &:hover,
    &:focus-visible {
      background: $color-surface-3;
      transform: scale(1.1);
      outline: none;
    }
  }
}

.list-inline-actions {
  flex: 0 0 auto;

  display: flex;
  align-items: center;
  justify-content: flex-end;

  gap: 0.3rem;

  &.hidden {
    display: none !important;
  }

  .list-action-btn {
    flex: 0 0 auto;

    min-height: 2rem;
    padding: 0.4rem 0.65rem;

    display: inline-flex;
    align-items: center;
    justify-content: center;

    gap: 0.35rem;

    background: transparent;
    border: none;
    border-radius: $radius-md;

    color: $color-text-muted;
    font-size: 0.78rem;
    font-weight: 500;

    white-space: nowrap;

    cursor: pointer;

    transition:
      color 0.13s ease,
      background 0.13s ease;

    &:hover,
    &:focus-visible {
      color: $color-text;
      background: $color-surface-3;
      outline: none;
    }

    &.danger {
      color: $color-error;

      &:hover,
      &:focus-visible {
        color: $color-error;
        background: color-mix(in srgb, $color-error, transparent 90%);
      }
    }
  }
}

.list-version-stats-row {
  min-width: 0;

  display: flex;
  align-items: center;

  margin-top: 0.1rem;
}

.list-stats-section {
  min-width: 0;

  display: flex;
  align-items: center;

  gap: 2rem;

  .list-meta-item {
    flex: 0 0 auto;

    display: flex;
    align-items: center;

    gap: 0.4rem;

    color: $color-text-muted;
    font-size: 0.8rem;
    white-space: nowrap;

    span {
      font-weight: 500;
    }
  }
}

/* ============================================================
   TOOLTIP
   ============================================================ */

.icon-tooltip-wrapper:hover .icon-tooltip,
.icon-tooltip-wrapper:focus-within .icon-tooltip {
  opacity: 1;
  pointer-events: auto;
  transform: none;
}

.icon-tooltip {
  position: absolute;
  left: -0.75rem;
  top: -1rem;
  z-index: 10;

  margin: 0;
  padding: 0.03em 0.28em;

  opacity: 0;
  pointer-events: none;

  border: 1px solid $color-surface-3;
  border-radius: $radius-md;
  background: $color-surface-1;

  color: $color-text;
  font-size: 0.7em;
  font-weight: 500;
  white-space: nowrap;

  box-shadow: 0 0.0625rem 0.125rem rgba(0, 0, 0, 0.06);

  transition: opacity 0.13s ease;
}

/* ============================================================
   STATES
   ============================================================ */

.empty-state {
  padding: 4rem 2rem;
  text-align: center;

  .empty-icon {
    margin-bottom: 1.5rem;
    color: $color-text-muted;
  }

  h3 {
    margin: 0 0 1rem;

    color: $color-text;
    font-size: 1.5rem;
    font-weight: 600;
  }

  p {
    margin: 0 0 2rem;

    color: $color-text-muted;
    font-size: 1rem;
  }
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;

  gap: 0.5rem;
  padding: 4rem 2rem;

  color: $color-text-muted;

  :global(.icon) {
    animation: spin 1s linear infinite;
  }
}

.error-message {
  display: flex;
  align-items: center;

  gap: 0.5rem;

  margin-bottom: 1rem;
  padding: 1rem;

  background: color-mix(in srgb, $color-error, transparent 90%);

  border: 1px solid $color-error;
  border-radius: $radius-md;

  color: $color-error;
}

/* ============================================================
   RESPONSIVE
   ============================================================ */

@media (max-width: 64rem) {
  .list-title-actions-row {
    gap: 1rem;
  }

  .list-inline-actions {
    gap: 0.1rem;

    .list-action-btn {
      padding-inline: 0.5rem;
    }
  }

  .list-stats-section {
    gap: 1.25rem;

    .list-meta-item {
      font-size: 0.75rem;
    }
  }
}

@media (max-width: 48rem) {
  .profiles-list {
    padding: 1rem;
  }

  .profiles-grid {
    grid-template-columns: 1fr;
  }

  .profile-card {
    padding: 1rem 0.8rem;
  }

  .profile-card.small {
    padding: 0.8rem;
  }

  .profile-main {
    grid-template-columns: 4.5rem minmax(0, 1fr);
    gap: 0.7rem;
  }

  .profile-actions {
    grid-template-columns: repeat(5, minmax(0, 1fr));

    button {
      font-size: 0.68rem;
    }
  }

  .list-item-main {
    flex-direction: column;
    align-items: stretch;

    gap: 0.75rem;
    padding: 0.75rem;
  }

  .list-item-icon-section {
    justify-content: center;

    .list-play-btn {
      flex: 1;
    }
  }

  .list-title-actions-row {
    align-items: flex-start;
    flex-direction: column;
    gap: 0.5rem;
  }

  .list-title {
    width: 100%;
  }

  .list-actions-section {
    width: 100%;
    justify-content: flex-start;
    flex-wrap: wrap;
  }

  .list-inline-actions {
    flex-wrap: wrap;
    justify-content: flex-start;
  }

  .list-version-stats-row {
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .list-stats-section {
    flex-wrap: wrap;
    gap: 0.75rem 1.5rem;
  }
}

@media (max-width: 32rem) {
  .profile-main {
    grid-template-columns: 1fr;
  }

  .profile-icon-column {
    flex-direction: row;
    justify-content: center;
  }

  .profile-icon-column > .profile-icon {
    width: 2.5rem;
    height: 2.5rem;
  }

  .play-below-icon {
    width: auto;
    min-width: 4.5rem;
  }

  .profile-meta-grid {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.18rem;
  }

  .meta-cell {
    justify-content: flex-start !important;
    text-align: left !important;
  }

  .meta-key,
  .meta-value {
    text-align: left !important;
  }

  .profile-actions {
    grid-template-columns: repeat(2, minmax(0, 1fr));

    button {
      font-size: 0.72rem;
    }
  }

  .list-title {
    flex-wrap: wrap;
  }

  .list-version {
    max-width: 100%;
  }

  .list-stats-section {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.35rem;
  }

  .list-version-stats-row {
    align-items: flex-start;
    flex-direction: column;
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}
</style>
