<script lang="ts">
import { createEventDispatcher } from 'svelte';
import { Icon } from '$lib';
import type { KableInstallation, ModInfoKind } from '$lib';

type ViewMode = 'grid' | 'list' | 'compact';

export let mod: ModInfoKind;
export let viewMode: ViewMode = 'grid';
export let currentInstallation: KableInstallation | null = null;
export let loading: boolean = false;
export let isInstalled: boolean = false;

const dispatch = createEventDispatcher<{
  downloadMod: { mod: ModInfoKind };
  infoMod: { mod: ModInfoKind };
}>();

// ModCard component for displaying mod information
function getModDisplayInfo(mod: ModInfoKind) {
  // Type guard for Rust enum format
  if ('Modrinth' in mod) {
    const modrinthData = mod.Modrinth;
    // Find the display version by matching latest_version ID with versions_obj
    let displayVersion = 'Unknown';
    
    // Debug: log what data we have
    console.log('Mod data for', modrinthData.title, ':', {
      latest_version: modrinthData.latest_version,
      versions: modrinthData.versions,
      versions_obj: modrinthData.versions_obj,
      hasVersionsObj: !!modrinthData.versions_obj
    });
    
    if (modrinthData.latest_version && modrinthData.versions_obj && modrinthData.versions_obj.length > 0) {
      // Find the version object that matches the latest_version ID
      const latestVersionObj = modrinthData.versions_obj.find(v => v.id === modrinthData.latest_version);
      if (latestVersionObj) {
        displayVersion = latestVersionObj.version_number;
        console.log('Found matching version object:', latestVersionObj.version_number);
      } else {
        // Fallback to last (newest) version's version_number if latest not found
        displayVersion = modrinthData.versions_obj[modrinthData.versions_obj.length - 1].version_number;
        console.log('Using last version object:', displayVersion);
      }
    } else if (modrinthData.versions && modrinthData.versions.length > 0) {
      // Fallback to last (newest) version ID from versions array if versions_obj not available
      displayVersion = modrinthData.versions[modrinthData.versions.length - 1];
      console.log('Using last version ID (fallback):', displayVersion);
    } else if (modrinthData.latest_version) {
      // Last resort: use the latest_version ID directly
      displayVersion = modrinthData.latest_version;
      console.log('Using latest_version directly:', displayVersion);
    }
    
    console.log('Final displayVersion for', modrinthData.title, ':', displayVersion);
    
    return {
      title: modrinthData.title || 'Unknown Mod',
      description: modrinthData.description || 'No description available',
      author: modrinthData.author || 'Unknown Author',
      icon_url: modrinthData.icon_url || null,
      downloads: modrinthData.downloads || 0,
      follows: modrinthData.follows || 0,
      updated: modrinthData.date_modified ? new Date(modrinthData.date_modified).toLocaleDateString() : 'Unknown',
      categories: modrinthData.categories || [],
      client_side: modrinthData.client_side || 'unknown',
      server_side: modrinthData.server_side || 'unknown',
      project_type: modrinthData.project_type || 'mod',
      latest_version: displayVersion
    };
  }
  
  // Type guard for TypeScript discriminated union format
  if ('kind' in mod && mod.kind === 'Modrinth') {
    const modrinthData = mod.data;
    // Find the display version by matching latest_version ID with versions_obj
    let displayVersion = 'Unknown';
    
    // Debug: log what data we have
    console.log('Mod data for', modrinthData.title, ':', {
      latest_version: modrinthData.latest_version,
      versions: modrinthData.versions,
      versions_obj: modrinthData.versions_obj,
      hasVersionsObj: !!modrinthData.versions_obj
    });
    
    if (modrinthData.latest_version && modrinthData.versions_obj && modrinthData.versions_obj.length > 0) {
      // Find the version object that matches the latest_version ID
      const latestVersionObj = modrinthData.versions_obj.find(v => v.id === modrinthData.latest_version);
      if (latestVersionObj) {
        displayVersion = latestVersionObj.version_number;
        console.log('Found matching version object:', latestVersionObj.version_number);
      } else {
        // Fallback to last (newest) version's version_number if latest not found
        displayVersion = modrinthData.versions_obj[modrinthData.versions_obj.length - 1].version_number;
        console.log('Using last version object:', displayVersion);
      }
    } else if (modrinthData.versions && modrinthData.versions.length > 0) {
      // Fallback to last (newest) version ID from versions array if versions_obj not available
      displayVersion = modrinthData.versions[modrinthData.versions.length - 1];
      console.log('Using last version ID (fallback):', displayVersion);
    } else if (modrinthData.latest_version) {
      // Last resort: use the latest_version ID directly
      displayVersion = modrinthData.latest_version;
      console.log('Using latest_version directly:', displayVersion);
    }
    
    console.log('Final displayVersion for', modrinthData.title, ':', displayVersion);
    
    return {
      title: modrinthData.title || 'Unknown Mod',
      description: modrinthData.description || 'No description available',
      author: modrinthData.author || 'Unknown Author',
      icon_url: modrinthData.icon_url || null,
      downloads: modrinthData.downloads || 0,
      follows: modrinthData.follows || 0,
      updated: modrinthData.date_modified ? new Date(modrinthData.date_modified).toLocaleDateString() : 'Unknown',
      categories: modrinthData.categories || [],
      client_side: modrinthData.client_side || 'unknown',
      server_side: modrinthData.server_side || 'unknown',
      project_type: modrinthData.project_type || 'mod',
      latest_version: displayVersion
    };
  }
  
  return {
    title: 'Unknown Mod',
    description: 'No description available',
    author: 'Unknown Author',
    icon_url: null,
    downloads: 0,
    follows: 0,
    updated: 'Unknown',
    categories: [],
    client_side: 'unknown',
    server_side: 'unknown',
    project_type: 'mod',
    latest_version: 'Unknown'
  };
}

// Filter categories for display
function getDisplayCategories(categories: string[]): string[] {
  const filterOut = ['forge', 'fabric', 'quilt', 'neoforge', 'client', 'server'];
  return categories
    .filter(cat => !filterOut.includes(cat.toLowerCase()))
    .slice(0, 4);
}

// Handle image errors
function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  const placeholder = img.nextElementSibling as HTMLElement;
  if (placeholder) {
    img.style.display = 'none';
    placeholder.style.display = 'flex';
  }
}

$: displayInfo = getModDisplayInfo(mod);

function handleDownload() {
  dispatch('downloadMod', { mod });
}

function handleInfo() {
  dispatch('infoMod', { mod });
}

function handleVersions() {
  // Navigate to the mod's versions page on Modrinth
  if ('Modrinth' in mod) {
    const url = `https://modrinth.com/mod/${mod.Modrinth.slug}/versions`;
    window.open(url, '_blank');
  } else if ('kind' in mod && mod.kind === 'Modrinth') {
    const url = `https://modrinth.com/mod/${mod.data.slug}/versions`;
    window.open(url, '_blank');
  }
}

function handleCardClick() {
  // Navigate to the mod's main page on Modrinth
  handleInfo();
}

function handleCardKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    handleCardClick();
  }
}
</script>

<div class="mod-card" class:installed={isInstalled} class:compact={viewMode === 'compact'} class:list={viewMode === 'list'} class:grid={viewMode === 'grid'}>
  {#if viewMode === 'compact'}
    <!-- Compact Mode -->
    <div class="compact-layout">
      <div class="compact-icon">
        {#if displayInfo.icon_url}
          <img 
            src={displayInfo.icon_url} 
            alt={displayInfo.title} 
            class="compact-mod-icon"
            on:error={handleImageError}
          />
          <div class="compact-icon-placeholder" style="display: none;">
            <Icon name="package" size="sm" />
          </div>
        {:else}
          <div class="compact-icon-placeholder">
            <Icon name="package" size="sm" />
          </div>
        {/if}
      </div>

      <div class="compact-info">
        <h4 class="compact-title" title={displayInfo.title}>
          {displayInfo.title}
        </h4>
        
        <div class="compact-meta">
          <span class="compact-downloads">
            <Icon name="download" size="sm" forceType="svg" />
            {displayInfo.downloads > 999 ? 
              (displayInfo.downloads / 1000000 > 1 ? 
                `${(displayInfo.downloads / 1000000).toFixed(1)}M` : 
                `${(displayInfo.downloads / 1000).toFixed(0)}K`) : 
              displayInfo.downloads.toLocaleString()}
          </span>
          
          {#if getDisplayCategories(displayInfo.categories).length > 0}
            <div class="compact-tags">
              {#each getDisplayCategories(displayInfo.categories).slice(0, 2) as category}
                <span class="compact-tag">{category}</span>
              {/each}
              {#if getDisplayCategories(displayInfo.categories).length > 2}
                <span class="compact-tag-more">+{getDisplayCategories(displayInfo.categories).length - 2}</span>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <div class="compact-actions">
        {#if !isInstalled && currentInstallation}
          <button 
            class="compact-download-btn" 
            on:click={handleDownload}
            disabled={loading}
          >
            <Icon name="download" size="sm" forceType="svg" />
            Download
          </button>
        {:else}
          <div class="compact-no-installation">
            <Icon name="info" size="sm" />
            Select installation
          </div>
        {/if}
        
        <button class="info-btn" on:click={handleInfo}>
          <Icon name="info" size="sm" />
        </button>
      </div>
    </div>
  {:else}
    <!-- Grid and List Mode -->
    {#if viewMode === 'grid'}
      <!-- Clickable grid card -->
      <div 
        class="mod-content-wrapper clickable"
        on:click={handleCardClick}
        on:keydown={handleCardKeydown}
        role="button"
        tabindex={0}
        aria-label={`View ${displayInfo.title} on Modrinth`}
      >
        <!-- New Flexbox Layout: Header + Data + Controls -->
        <div class="flex-layout">
          <!-- Header Section -->
          <div class="flex-header">
            <div class="flex-icon">
              {#if displayInfo.icon_url}
                <img 
                  src={displayInfo.icon_url} 
                  alt={displayInfo.title} 
                  class="flex-icon-img"
                  on:error={handleImageError}
                />
                <div class="flex-icon-placeholder" style="display: none;">
                  <Icon name="package" size="md" />
                </div>
              {:else}
                <div class="flex-icon-placeholder">
                  <Icon name="package" size="md" />
                </div>
              {/if}
            </div>

            <div class="flex-header-info">
              <h3 class="flex-title" title={`${displayInfo.title} - Latest: ${displayInfo.latest_version}`}>
                {displayInfo.title}
              </h3>
              <div class="flex-author">
                by {displayInfo.author}
              </div>
            </div>

            <div class="flex-controls">
              <button 
                class="control-btn versions-btn" 
                on:click|stopPropagation={handleVersions}
                title="View all versions"
              >
                <Icon name="list" size="sm" />
              </button>
              {#if !isInstalled}
                {#if currentInstallation}
                  <button 
                    class="control-btn download-btn" 
                    on:click|stopPropagation={handleDownload}
                    disabled={loading}
                    title="Download latest version"
                  >
                    <Icon name="download" size="sm" forceType="svg" />
                  </button>
                {:else}
                  <button class="control-btn disabled-btn" disabled title="Select installation first">
                    <Icon name="info" size="sm" />
                  </button>
                {/if}
              {/if}
            </div>
          </div>

          <!-- Data Section -->
          <div class="flex-data">
            <div class="flex-stats">
              <div class="flex-stat">
                <Icon name="download" size="sm" forceType="svg" />
                <span class="stat-value">
                  {displayInfo.downloads > 999 ? 
                    (displayInfo.downloads / 1000000 > 1 ? 
                      `${(displayInfo.downloads / 1000000).toFixed(1)}M` : 
                      `${(displayInfo.downloads / 1000).toFixed(0)}K`) : 
                    displayInfo.downloads.toLocaleString()}
                </span>
              </div>

              <div class="flex-stat">
                <Icon name="star" size="sm" forceType="svg" />
                <span class="stat-value">{displayInfo.follows.toLocaleString()}</span>
              </div>

              <div class="flex-stat">
                <Icon name="calendar" size="sm" forceType="svg" />
                <span class="stat-value">{displayInfo.updated}</span>
              </div>
            </div>

            <div class="flex-description">
              {displayInfo.description}
            </div>

            {#if getDisplayCategories(displayInfo.categories).length > 0}
              <div class="flex-tags">
                {#each getDisplayCategories(displayInfo.categories) as category}
                  <span class="flex-tag">{category}</span>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    {:else}
      <!-- Non-clickable list card -->
      <div class="mod-content-wrapper">
        <!-- List Mode (unchanged) -->
        <div class="mod-header-section">
          <div class="mod-icon">
            {#if displayInfo.icon_url}
              <img 
                src={displayInfo.icon_url} 
                alt={displayInfo.title} 
                class="mod-icon-img"
                on:error={handleImageError}
              />
              <div class="mod-icon-placeholder" style="display: none;">
                <Icon name="package" size="md" />
              </div>
            {:else}
              <div class="mod-icon-placeholder">
                <Icon name="package" size="md" />
              </div>
            {/if}
          </div>

          <div class="mod-header-info">
            <h3 class="mod-title" title={displayInfo.title}>
              {displayInfo.title}
            </h3>
            
            <div class="mod-author">
              by {displayInfo.author}
            </div>
            
            {#if getDisplayCategories(displayInfo.categories).length > 0}
              <div class="mod-tags">
                {#each getDisplayCategories(displayInfo.categories) as category}
                  <span class="mod-tag">{category}</span>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <div class="mod-stats">
          <div class="mod-stat">
            <Icon name="download" size="sm" forceType="svg" />
            <span class="stat-value">
              {displayInfo.downloads > 999 ? 
                (displayInfo.downloads / 1000000 > 1 ? 
                  `${(displayInfo.downloads / 1000000).toFixed(1)}M` : 
                  `${(displayInfo.downloads / 1000).toFixed(0)}K`) : 
                displayInfo.downloads.toLocaleString()}
            </span>
            <span class="stat-label">Downloads</span>
          </div>

          <div class="mod-stat">
            <Icon name="star" size="sm" forceType="svg" />
            <span class="stat-value">{displayInfo.follows.toLocaleString()}</span>
            <span class="stat-label">Follows</span>
          </div>

          <div class="mod-stat">
            <Icon name="calendar" size="sm" forceType="svg" />
            <span class="stat-value">{displayInfo.updated}</span>
            <span class="stat-label">Updated</span>
          </div>
        </div>

        <div class="mod-content-section">
          <p class="mod-description">
            {displayInfo.description}
          </p>
        </div>

        <div class="mod-actions">
          {#if isInstalled}
            <div class="installed-indicator">
              Installed
            </div>
          {:else if currentInstallation}
            <button 
              class="download-btn" 
              on:click={handleDownload}
              disabled={loading}
            >
              <Icon name="download" size="sm" />
              Download
            </button>
          {:else}
            <div class="no-installation-warning">
              <Icon name="info" size="sm" />
              <span>Select installation first</span>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

.mod-card {
  background: $card;
  border: 1px solid rgba($primary, 0.08);
  border-radius: 0.5rem;
  overflow: hidden;
  transition: all 0.2s ease;
  display: flex;
  position: relative;
  
  &.installed {
    background: linear-gradient(135deg, rgba($green, 0.05) 0%, rgba($green, 0.02) 100%);
    border-color: rgba($green, 0.3);
  }
  
  &.grid {
    flex-direction: column;
    width: 100%;
    height: auto;
    overflow: hidden;
    
    &:hover {
      transform: translateY(-2px);
      box-shadow: 0 8px 25px rgba($dark-900, 0.15);
      background: rgba($primary, 0.02);
      border-color: rgba($primary, 0.15);
    }
  }
  
  &.list {
    flex-direction: row;
    width: 100%;
    max-width: none;
    min-height: fit-content;

    &:hover {
      background: rgba($primary, 0.02);
      border-color: rgba($primary, 0.12);
      transform: none;
      box-shadow: 0 2px 8px rgba($dark-900, 0.08);
    }
  }
  
  &.compact {
    border-radius: 0.375rem;
    height: fit-content;
    flex: 1 1 220px;
    min-width: 220px;
    max-width: 300px;
    
    &:hover {
      background: rgba($primary, 0.03);
      border-color: rgba($primary, 0.2);
      transform: none;
      box-shadow: 0 2px 6px rgba($dark-900, 0.1);
    }
  }
  
  .mod-content-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  &.list .mod-content-wrapper {
    flex-direction: row;
    align-items: flex-start;
    gap: 0.5rem;
  }
  
  .mod-header-section {
    display: flex;
    align-items: flex-start;
    gap: 0.25rem;
  }
  
  &.list .mod-header-section {
    flex: 0 0 auto;
    display: flex;
    gap: 1rem;
    align-items: flex-start;
    min-width: 300px;
  }
  
  .mod-header-info {
    flex: 1;
    min-width: 0;
    
    .mod-title {
      margin: 0 0 0.25rem 0;
      font-size: 0.85em;
      font-weight: 600;
      color: $text;
      line-height: 1.2;
      word-wrap: break-word;
      overflow-wrap: break-word;
      hyphens: auto;
      display: -webkit-box;
      -webkit-line-clamp: 2;
      line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }
    
    .mod-author {
      color: $placeholder;
      font-size: 0.8em;
      margin-bottom: 0.375rem;
    }
    
    .mod-tags {
      display: flex;
      flex-wrap: wrap;
      gap: 0.25rem;
      
      .mod-tag {
        background: rgba($primary, 0.08);
        color: $primary;
        padding: 0.1875rem 0.375rem;
        border-radius: 0.25rem;
        font-size: 0.7em;
        font-weight: 500;
        text-transform: capitalize;
      }
    }
  }
  
  &.list .mod-header-info .mod-title {
    font-size: 1.125rem;
    margin-bottom: 0.5rem;
  }
  
  .mod-icon {
    width: 48px;
    height: 48px;
    border-radius: 0.375rem;
    overflow: hidden;
    background: rgba($placeholder, 0.1);
    border: 1px solid rgba($placeholder, 0.2);
    flex-shrink: 0;
    position: relative;
    
    .mod-icon-img {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }
    
    .mod-icon-placeholder {
      width: 100%;
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: $placeholder;
      background: rgba($placeholder, 0.05);
    }
  }
  
  .mod-stats {
    display: flex;
    gap: 1rem;
    margin: 0.5rem 0;
    flex-direction: column;
    
    .mod-stat {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      color: $placeholder;
      font-size: 0.8em;
      
      .stat-value {
        font-weight: 600;
        color: $text;
      }
      
      .stat-label {
        color: $placeholder;
        font-size: 0.9em;
      }
    }
  }
  
  &.list .mod-stats {
    flex-direction: row;
    gap: 1.5rem;
    margin: 0.75rem 0;
  }
  
  .mod-content-section {
    flex: 1;
    
    .mod-description {
      color: $placeholder;
      font-size: 0.85em;
      line-height: 1.4;
      margin: 0 0 0.75rem 0;
      display: -webkit-box;
      -webkit-line-clamp: 3;
      line-clamp: 3;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }
  }
  
  &.list .mod-content-section {
    flex: 1;
    max-width: none;
    
    .mod-description {
      -webkit-line-clamp: 2;
      line-clamp: 2;
    }
  }
  
  .mod-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: auto;
    padding-top: 0.75rem;
    
    .download-btn {
      flex: 1;
      background: $primary;
      color: white;
      border: none;
      border-radius: 0.375rem;
      padding: 0.75rem 1rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.2s ease;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.375rem;
      
      &:hover:not(:disabled) {
        background: rgba($primary, 0.8);
        transform: translateY(-1px);
      }
      
      &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
        transform: none;
      }
    }
    
    .no-installation-warning {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.375rem;
      padding: 0.75rem 1rem;
      border: 1px solid rgba($yellow, 0.3);
      border-radius: 0.375rem;
      background: rgba($yellow, 0.05);
      color: $yellow;
      font-size: 0.85em;
      font-weight: 500;
    }
    
    .installed-indicator {
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 0.5rem 1rem;
      border: 1px solid rgba($green, 0.4);
      border-radius: 0.375rem;
      background: linear-gradient(135deg, rgba($green, 0.15) 0%, rgba($green, 0.08) 100%);
      color: $green;
      font-weight: 600;
      font-size: 0.8em;
      white-space: nowrap;
      
      :global(.icon) {
        color: $green;
      }
    }
  }
  
  &.grid .mod-actions {
    width: 100%;
    justify-content: center;
  }

  &.list .mod-actions {
    margin-left: auto;
    flex-shrink: 0;
    padding-top: 0;
    min-width: 200px;
  }
}

// New Flexbox Layout Styles
.flex-layout {
  display: flex;
  flex-direction: column;
  padding: 0.75rem;
  height: 100%;
  gap: 0.5rem;
  
  .flex-header {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    
    .flex-icon {
      width: 48px;
      height: 48px;
      border-radius: 0.375rem;
      overflow: hidden;
      background: rgba($placeholder, 0.1);
      border: 1px solid rgba($placeholder, 0.2);
      flex-shrink: 0;
      position: relative;
      
      .flex-icon-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
      }
      
      .flex-icon-placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        color: $placeholder;
        background: rgba($placeholder, 0.05);
      }
    }
    
    .flex-header-info {
      flex: 1;
      min-width: 0;
      
      .flex-title {
        margin: 0 0 0.25rem 0;
        font-size: 0.9rem;
        font-weight: 600;
        color: $text;
        line-height: 1.3;
        word-wrap: break-word;
        overflow-wrap: break-word;
        hyphens: auto;
        white-space: normal;
      }
      
      .flex-author {
        color: $placeholder;
        font-size: 0.75em;
        font-weight: 500;
      }
    }
    
    .flex-controls {
      display: flex;
      flex-direction: column;
      gap: 0.375rem;
      align-items: center;
      flex-shrink: 0;
      
      .control-btn {
        width: 32px;
        height: 32px;
        padding: unset;
        border: none;
        border-radius: 0.375rem;
        cursor: pointer;
        transition: all 0.2s ease;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
        
        &.download-btn {
          background: $primary;
          color: white;
          
          &:hover:not(:disabled) {
            background: rgba($primary, 0.8);
            transform: translateY(-1px);
          }
          
          &:disabled {
            opacity: 0.6;
            cursor: not-allowed;
            transform: none;
          }
        }
        
        &.versions-btn {
          background: rgba($secondary, 0.1);
          color: $secondary;
          border: 1px solid rgba($secondary, 0.3);
          
          &:hover {
            background: rgba($secondary, 0.2);
            border-color: rgba($secondary, 0.5);
          }
        }
        
        &.disabled-btn {
          background: rgba($placeholder, 0.05);
          color: $placeholder;
          border: 1px solid rgba($placeholder, 0.2);
          cursor: not-allowed;
          opacity: 0.6;
        }
      }
    }
  }
  
  .flex-data {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    
    .flex-stats {
      display: flex;
      gap: 0.5rem;
      
      .flex-stat {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        color: $placeholder;
        font-size: 0.65em;
        
        .stat-value {
          font-weight: 600;
          color: $text;
        }
      }
    }
    
    .flex-description {
      color: $placeholder;
      font-size: 0.75em;
      line-height: 1.4;
      display: -webkit-box;
      -webkit-line-clamp: 2;
      line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
      word-wrap: break-word;
      overflow-wrap: break-word;
    }
    
    .flex-tags {
      display: flex;
      flex-wrap: wrap;
      gap: 0.25rem;
      
      .flex-tag {
        background: rgba($primary, 0.08);
        color: $primary;
        padding: 0.125rem 0.25rem;
        border-radius: 0.1875rem;
        font-size: 0.65em;
        font-weight: 500;
        text-transform: capitalize;
      }
    }
  }
}

.clickable {
  cursor: pointer;
}

// Compact layout specific styles  
.compact-layout {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 0.5rem;
  padding: 0.5rem;
  position: relative;
  
  .compact-icon {
    width: 28px;
    height: 28px;
    border-radius: 0.25rem;
    overflow: hidden;
    background: rgba($placeholder, 0.1);
    border: 1px solid rgba($placeholder, 0.2);
    flex-shrink: 0;
    position: relative;
    
    .compact-mod-icon {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }
    
    .compact-icon-placeholder {
      width: 100%;
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: $placeholder;
      background: rgba($placeholder, 0.05);
    }
  }
  
  .compact-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.0625rem;
    
    .compact-title {
      margin: 0;
      font-size: 0.8em;
      font-weight: 600;
      color: $text;
      line-height: 1.2;
      word-wrap: break-word;
      overflow-wrap: break-word;
      hyphens: auto;
      display: -webkit-box;
      -webkit-line-clamp: 2;
      line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }
    
    .compact-meta {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      flex-wrap: wrap;
      
      .compact-downloads {
        display: flex;
        align-items: center;
        gap: 0.1875rem;
        color: $placeholder;
        font-size: 0.65em;
        font-weight: 500;
      }
      
      .compact-tags {
        display: flex;
        gap: 0.1875rem;
        flex-wrap: wrap;
        
        .compact-tag {
          background: rgba($primary, 0.08);
          color: $primary;
          padding: 0.09375rem 0.25rem;
          border-radius: 0.1875rem;
          font-size: 0.55em;
          font-weight: 500;
          text-transform: capitalize;
        }
        
        .compact-tag-more {
          color: $placeholder;
          font-size: 0.55em;
          font-weight: 500;
        }
      }
    }
  }
  
  .compact-actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    flex-shrink: 0;
    
    .compact-download-btn {
      background: $primary;
      color: white;
      border: none;
      border-radius: 0.25rem;
      padding: 0.375rem;
      cursor: pointer;
      transition: all 0.2s ease;
      display: flex;
      align-items: center;
      justify-content: center;
      
      &:hover:not(:disabled) {
        background: rgba($primary, 0.8);
        transform: translateY(-1px);
      }
      
      &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
        transform: none;
      }
    }
    
    .compact-no-installation {
      display: flex;
      align-items: center;
      gap: 0.25rem;
      color: $placeholder;
      font-size: 0.75em;
      opacity: 0.6;
    }
    
    .info-btn {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 32px;
      height: 32px;
      border: 1px solid rgba($placeholder, 0.3);
      border-radius: 0.375rem;
      background: rgba($placeholder, 0.05);
      color: $placeholder;
      cursor: pointer;
      transition: all 0.2s ease;
      
      &:hover {
        background: rgba($secondary, 0.1);
        color: $secondary;
        border-color: rgba($secondary, 0.3);
      }
    }
  }
}
</style>
