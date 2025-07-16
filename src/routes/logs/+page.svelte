<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { 
    Icon, 
    logsService, 
    gameInstances, 
    currentLogs, 
    selectedInstanceId, 
    type GameInstance 
  } from '$lib';

  let selectedLogType: 'launcher' | 'game' = 'launcher';
  let logContainer: HTMLElement;
  let autoScroll = true; // Ensure auto-scroll is enabled by default
  let searchTerm = '';
  
  // Log level filters (all enabled by default except debug)
  let logLevelFilters = {
    error: true,
    warn: true,
    info: true,
    debug: false
  };
  
  let showLogLevelDropdown = false;

  onMount(async () => {
    // Logs service is already initialized in the layout
    // No need to initialize again here
    
    // Add event listener for clicking outside dropdown
    document.addEventListener('click', handleClickOutside);
  });

  onDestroy(() => {
    // Keep logs service running since it's used globally
    // Remove event listener
    document.removeEventListener('click', handleClickOutside);
  });

  function selectInstance(instanceId: string | 'global') {
    selectedInstanceId.set(instanceId);
    // Reset to launcher logs when switching instances
    selectedLogType = 'launcher';
  }

  function getInstanceDisplayName(instance: GameInstance): string {
    if (!instance || !instance.launchedAt) return 'Unknown';
    
    const launchedAt = instance.launchedAt instanceof Date 
      ? instance.launchedAt 
      : new Date(instance.launchedAt);
    
    const duration = Math.floor((Date.now() - launchedAt.getTime()) / 1000);
    const durationStr = duration < 60 ? `${duration}s` : `${Math.floor(duration / 60)}m`;
    return `${instance.profileName || 'Unknown'} (${durationStr})`;
  }

  function getStatusIcon(status: GameInstance['status']): string {
    switch (status) {
      case 'launching': return 'rocket';
      case 'running': return 'play';
      case 'completed': return 'check';
      case 'crashed': return 'alert';
      case 'stopped': return 'square'; // Better icon for stopped
      default: return 'help';
    }
  }

  function getStatusColor(status: GameInstance['status']): string {
    switch (status) {
      case 'launching': return 'warning';
      case 'running': return 'success';
      case 'completed': return 'info';
      case 'crashed': return 'danger';
      case 'stopped': return 'secondary'; // Changed from 'muted' for better visibility
      default: return 'muted';
    }
  }

  function formatTime(date: Date | string): string {
    const dateObj = date instanceof Date ? date : new Date(date);
    return dateObj.toLocaleTimeString('en-US', { hour12: false });
  }

  function formatLogLevel(level: string): string {
    return level.toUpperCase();
  }

  function getLogLevelIcon(level: string): string {
    switch (level.toLowerCase()) {
      case 'error': return 'alert';
      case 'warn': return 'warning';
      case 'info': return 'info';
      case 'debug': return 'bug';
      default: return 'message';
    }
  }

  function getLogLevelClass(level: string): string {
    switch (level.toLowerCase()) {
      case 'error': return 'danger';
      case 'warn': return 'warning';
      case 'info': return 'info';
      case 'debug': return 'muted';
      default: return 'secondary';
    }
  }

  async function clearCurrentLogs() {
    const instanceId = $selectedInstanceId;
    await logsService.clearLogs(instanceId === 'global' ? undefined : instanceId);
  }

  async function exportCurrentLogs() {
    const instanceId = $selectedInstanceId;
    await logsService.exportLogs(instanceId === 'global' ? undefined : instanceId);
  }

  async function copyLogEntry(logEntry: any) {
    if (!logEntry) return;
    
    const timestamp = logEntry.timestamp instanceof Date 
      ? logEntry.timestamp 
      : new Date(logEntry.timestamp);
      
    const logText = `[${formatTime(timestamp)}] ${formatLogLevel(logEntry.level || 'info')} ${logEntry.message || ''}`;
    try {
      await navigator.clipboard.writeText(logText);
      // Could add a toast notification here if desired
    } catch (err) {
      console.error('Failed to copy log entry:', err);
    }
  }

  function scrollToBottom() {
    if (logContainer && autoScroll) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  }

  function handleScroll() {
    if (logContainer) {
      const { scrollTop, scrollHeight, clientHeight } = logContainer;
      autoScroll = scrollTop + clientHeight >= scrollHeight - 10;
    }
  }

  function toggleLogLevelDropdown() {
    showLogLevelDropdown = !showLogLevelDropdown;
  }

  function getLogLevelDisplayName(level: string): string {
    switch (level) {
      case 'error': return 'Errors';
      case 'warn': return 'Warnings';
      case 'info': return 'Info';
      case 'debug': return 'Debug';
      default: return level;
    }
  }

  function getEnabledLogLevelsCount(): number {
    return Object.values(logLevelFilters).filter(Boolean).length;
  }

  // Close dropdown when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.log-level-dropdown')) {
      showLogLevelDropdown = false;
    }
  }

  $: sortedInstances = $gameInstances ? Array.from($gameInstances.values()).sort((a: GameInstance, b: GameInstance) => {
    const aTime = a.launchedAt instanceof Date ? a.launchedAt.getTime() : new Date(a.launchedAt).getTime();
    const bTime = b.launchedAt instanceof Date ? b.launchedAt.getTime() : new Date(b.launchedAt).getTime();
    return bTime - aTime;
  }) : [];

  $: currentLogsData = $currentLogs || { launcherLogs: [], gameLogs: [] };
  $: activeLogEntries = selectedLogType === 'launcher' 
    ? (currentLogsData.launcherLogs || [])
    : (currentLogsData.gameLogs || []);

  // Filter logs based on search and log level filters
  $: filteredLogs = (activeLogEntries || []).filter(log => {
    if (!log) return false;
    const matchesSearch = !searchTerm || 
      (log.message && log.message.toLowerCase().includes(searchTerm.toLowerCase()));
    
    // Check if the log level is enabled in filters
    const logLevel = (log.level || 'info').toLowerCase();
    const matchesLevelFilter = logLevel in logLevelFilters ? 
      logLevelFilters[logLevel as keyof typeof logLevelFilters] : 
      true; // Show unknown log levels by default
    
    return matchesSearch && matchesLevelFilter;
  });

  // Auto-scroll when new logs arrive
  $: if (filteredLogs && filteredLogs.length > 0 && autoScroll) {
    setTimeout(() => scrollToBottom(), 50);
  }

  // Check if any filters are active
  $: hasActiveFilters = searchTerm || getEnabledLogLevelsCount() < 4;
</script>

<div class="logs-page">
  <!-- Page Header -->
  <div class="page-header">
    <div class="header-content">
      <h1>
        <Icon name="archive" size="md" />
        Logs
      </h1>
      <p>Monitor launcher activity and game processes in real-time</p>
      {#if (sortedInstances || []).length > 0}
        <div class="stat-badge">
          <Icon name="activity" size="sm" />
          {(sortedInstances || []).length} active instance{(sortedInstances || []).length !== 1 ? 's' : ''}
        </div>
      {/if}
    </div>
    <div class="header-actions">
      <button 
        class="btn btn-danger btn-sm"
        on:click={clearCurrentLogs}
        title="Clear current logs"
      >
        <Icon name="trash" size="sm" />
        Clear
      </button>
      <button 
        class="btn btn-secondary btn-sm"
        on:click={exportCurrentLogs}
        title="Export logs to file"
      >
        <Icon name="download" size="sm" />
        Export
      </button>
    </div>
  </div>

  <!-- Filters Section -->
  <div class="filters-section">
    <div class="search-container">
      <Icon name="search" size="sm" />
      <input
        type="text"
        placeholder="Search logs..."
        bind:value={searchTerm}
        class="search-input"
      />
    </div>
    <div class="filter-controls">
      <div class="log-level-dropdown">
        <button 
          class="dropdown-trigger"
          on:click={toggleLogLevelDropdown}
          type="button"
        >
          <span>Log Levels ({getEnabledLogLevelsCount()}/4)</span>
          <Icon name={showLogLevelDropdown ? 'chevron-up' : 'chevron-down'} size="sm" />
        </button>
        
        {#if showLogLevelDropdown}
          <div class="dropdown-menu">
            <div class="dropdown-header">
              <span>Select log levels to display</span>
            </div>
            {#each Object.entries(logLevelFilters) as [level, enabled]}
              <label class="dropdown-item">
                <input
                  type="checkbox"
                  checked={enabled}
                  on:change={(e) => {
                    const target = e.target as HTMLInputElement;
                    logLevelFilters[level as keyof typeof logLevelFilters] = target.checked;
                  }}
                />
                <Icon name={getLogLevelIcon(level)} size="sm" />
                <span>{getLogLevelDisplayName(level)}</span>
              </label>
            {/each}
          </div>
        {/if}
      </div>
      
      <label class="checkbox-label">
        <input
          type="checkbox"
          bind:checked={autoScroll}
        />
        <span>Auto-scroll</span>
      </label>
    </div>
  </div>

  <!-- Instance Tabs -->
  <div class="tabs-container">
    <div class="tab-list">
      <!-- Global Tab -->
      <button
        class="tab-button {$selectedInstanceId === 'global' ? 'active' : ''}"
        on:click={() => selectInstance('global')}
      >
        <Icon name="globe" size="sm" />
        <span>Launcher</span>
      </button>

      <!-- Instance Tabs -->
      {#each sortedInstances as instance (instance.id)}
        <button
          class="tab-button {$selectedInstanceId === instance.id ? 'active' : ''}"
          on:click={() => selectInstance(instance.id)}
        >
          <Icon name={getStatusIcon(instance.status)} size="sm" />
          <span>{getInstanceDisplayName(instance)}</span>
          <span class="status-badge {getStatusColor(instance.status)}">
            {instance.status}
          </span>
        </button>
      {/each}
    </div>
  </div>

  <!-- Sub-tabs for Log Types (when instance is selected) -->
  {#if $selectedInstanceId !== 'global'}
    <div class="sub-tabs-container">
      <div class="sub-tab-list">
        <button
          class="sub-tab-button {selectedLogType === 'launcher' ? 'active' : ''}"
          on:click={() => selectedLogType = 'launcher'}
        >
          <Icon name="rocket" size="sm" />
          <span>Launcher</span>
          <span class="count-badge">{(currentLogsData.launcherLogs || []).length}</span>
        </button>
        <button
          class="sub-tab-button {selectedLogType === 'game' ? 'active' : ''}"
          on:click={() => selectedLogType = 'game'}
        >
          <Icon name="gamepad" size="sm" />
          <span>Game</span>
          <span class="count-badge">{(currentLogsData.gameLogs || []).length}</span>
        </button>
      </div>
    </div>
  {/if}

  <!-- Log Content -->
  <div class="log-content">
    <div
      bind:this={logContainer}
      on:scroll={handleScroll}
      class="log-container"
    >
      {#if filteredLogs.length === 0}
        <div class="empty-state">
          <div class="empty-icon">
            {#if hasActiveFilters}
              <Icon name="search" size="xl" />
            {:else}
              <Icon name="archive" size="xl" />
            {/if}
          </div>
          <h3>
            {#if hasActiveFilters}
              No logs match your filters
            {:else}
              No logs yet
            {/if}
          </h3>
          <p>
            {#if hasActiveFilters}
              Try adjusting your search or filter settings
            {:else}
              Launch an installation to see logs here
            {/if}
          </p>
        </div>
      {:else}
        <div class="log-entries">
          {#each filteredLogs as logEntry, index (logEntry?.timestamp ? (logEntry.timestamp instanceof Date ? logEntry.timestamp.getTime() : new Date(logEntry.timestamp).getTime()) + '-' + index : 'unknown-' + index)}
            {#if logEntry}
              <div class="log-entry">
                <div 
                  class="log-copy-icon"
                  on:click={() => copyLogEntry(logEntry)}
                  title="Copy log entry"
                  role="button"
                  tabindex="0"
                  on:keydown={(e) => e.key === 'Enter' && copyLogEntry(logEntry)}
                >
                  <Icon name="clipboard" size="sm" />
                </div>
                <div class="log-timestamp">
                  {formatTime(logEntry.timestamp)}
                </div>
                <div class="log-level badge {getLogLevelClass(logEntry.level || 'info')}">
                  <Icon name={getLogLevelIcon(logEntry.level || 'info')} size="sm" />
                  {formatLogLevel(logEntry.level || 'info')}
                </div>
                <div class="log-message">
                  {logEntry.message || ''}
                </div>
              </div>
            {/if}
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Status Bar -->
  <div class="status-bar">
    <div class="status-left">
      <span class="status-text">
        {#if $selectedInstanceId === 'global'}
          Global logs: {(currentLogsData.launcherLogs || []).length} entries
        {:else}
          {selectedLogType === 'launcher' ? 'Launcher' : 'Game'} logs: {(activeLogEntries || []).length} entries
        {/if}
        {#if hasActiveFilters}
          (filtered: {(filteredLogs || []).length})
        {/if}
      </span>
      {#if !autoScroll}
        <button
          class="btn btn-link btn-sm"
          on:click={scrollToBottom}
        >
          <Icon name="arrow-down" size="sm" />
          Jump to bottom
        </button>
      {/if}
    </div>
    <div class="status-right">
      <span class="status-text">{(sortedInstances || []).length} active instances</span>
      {#if autoScroll}
        <span class="auto-scroll-indicator">
          <Icon name="refresh" size="sm" />
          Auto-scroll
        </span>
      {/if}
    </div>
  </div>
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;

  .logs-page {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1.5rem;
    
    .header-content {
      flex: 1;
      
      h1 {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin: 0 0 0.5rem 0;
        font-size: 2rem;
        font-weight: 700;
      }
      
      p {
        margin: 0 0 1rem 0;
        color: $placeholder;
        font-size: 1.1rem;
      }
      
      .stat-badge {
        display: inline-flex;
        align-items: center;
        gap: 0.25rem;
        padding: 0.25rem 0.75rem;
        background: rgba($primary, 0.1);
        color: $primary;
        border-radius: $border-radius;
        font-size: 0.875rem;
        font-weight: 500;
      }
    }
    
    .header-actions {
      display: flex;
      gap: 0.5rem;
    }
  }

  .filters-section {
    display: flex;
    gap: 1rem;
    align-items: center;
    margin-bottom: 0.5rem;
    padding: 1rem;
    background: $container;
    border-radius: $border-radius;
    border: 1px solid $dark-200;

    .search-container {
      flex: 1;
      position: relative;
      display: flex;
      align-items: center;
      gap: 0.5rem;
      
      .search-input {
        flex: 1;
        padding: 0.5rem 0.75rem;
        font-size: 0.9rem;
        
        &:focus {
          outline: none;
          border-color: $primary;
        }
        
        &::placeholder {
          color: $placeholder;
        }
      }
    }
    
    .filter-controls {
      display: flex;
      gap: 1rem;
      align-items: center;
      
      .log-level-dropdown {
        position: relative;
        
        .dropdown-trigger {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 0.75rem;
          background: $card;
          border: 1px solid $dark-200;
          border-radius: $border-radius-small;
          color: $text;
          font-size: 0.9rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s ease;
          white-space: nowrap;
          
          &:hover {
            background: $dark-200;
            border-color: $primary;
          }
          
          &:focus {
            outline: none;
            border-color: $primary;
          }
        }
        
        .dropdown-menu {
          position: absolute;
          top: calc(100% + 0.25rem);
          right: 0;
          min-width: 180px;
          background: $container;
          border: 1px solid $dark-200;
          border-radius: $border-radius;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
          z-index: 1000;
          overflow: hidden;
          
          .dropdown-header {
            padding: 0.75rem;
            background: $card;
            border-bottom: 1px solid $dark-200;
            font-size: 0.85rem;
            font-weight: 600;
            color: $placeholder;
          }
          
          .dropdown-item {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.5rem 0.75rem;
            font-size: 0.9rem;
            cursor: pointer;
            transition: background-color 0.2s ease;
            
            &:hover {
              background: $card;
            }
            
            input[type="checkbox"] {
              accent-color: $primary;
            }
          }
        }
      }
      
      .checkbox-label {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.9rem;
        cursor: pointer;
        
        input[type="checkbox"] {
          accent-color: $primary;
        }
      }
    }
  }

  .tabs-container {
    margin-bottom: 1rem;
    
    .tab-list {
      display: flex;
      background: $container;
      border-radius: $border-radius;
      border: 1px solid $dark-200;
      overflow-x: auto;
      
      .tab-button {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.75rem 1rem;
        background: transparent;
        border: none;
        color: $placeholder;
        font-size: 0.9rem;
        font-weight: 500;
        cursor: pointer;
        white-space: nowrap;
        transition: all 0.2s ease;
        border-right: 1px solid $dark-200;

        &:last-child {
          border-right: none;
        }
        
        &:hover {
          // background: var(--hover);
          color: $text;
        }
        
        &.active {
          background: $primary;
          color: $text;
        }
        
        .status-badge {
          padding: 0.125rem 0.375rem;
          border-radius: calc($border-radius * 0.5);
          font-size: 0.85rem;
          font-weight: 600;
          
          &.success { background: rgba($green, 0.1); color: $green; }
          &.warning { background: rgba($yellow, 0.1); color: $yellow; }
          &.danger { background: rgba($red, 0.1); color: $red; }
          &.info { background: rgba($blue, 0.1); color: $blue; }
          &.secondary { background: rgba($text, 0.2); color: $text; }
          &.muted { background: rgba($dark-300, 0.1); color: white; }
        }
      }
    }
  }

  .sub-tabs-container {
    margin-bottom: 1rem;
    
    .sub-tab-list {
      display: flex;
      background: $container;
      border-radius: $border-radius;
      border: 1px solid $dark-200;
      
      .sub-tab-button {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.25rem 0.75rem;
        background: transparent;
        border: none;
        color: $placeholder;
        font-size: 0.875rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
        border-right: 1px solid $dark-200;
        border-radius: $border-radius-small;
        
        &:last-child {
          border-right: none;
        }
        
        &:hover {
          background: $card;
          color: $text;
        }
        
        &.active {
          background: $primary;
          color: white;
        }
        
        .count-badge {
          padding: 0.125rem 0.25rem;
          background: rgba(255, 255, 255, 0.2);
          border-radius: calc($border-radius * 0.5);
          font-size: 0.75rem;
          min-width: 1.25rem;
          text-align: center;
        }
      }
    }
  }

  .log-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    
    .log-container {
      flex: 1;
      overflow-y: auto;
      background: $container;
      border: 1px solid $dark-200;
      border-radius: $border-radius;
      margin-bottom: 0.5rem;
      
      .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        padding: 2rem;
        text-align: center;
        color: $placeholder;
        
        .empty-icon {
          margin-bottom: 1rem;
          opacity: 0.5;
        }
        
        h3 {
          margin: 0 0 0.5rem 0;
          font-size: 1.25rem;
          font-weight: 600;
        }
        
        p {
          margin: 0;
          font-size: 0.9rem;
        }
      }
      
      .log-entries {
        padding: 0.25rem;
        
        .log-entry {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.05rem;
          margin-bottom: 0;
          border-radius: $border-radius;
          transition: background-color 0.2s ease;
          
          &:hover {
            background: $card;
            
            .log-copy-icon {
              opacity: 1;
            }
          }
          
          .log-copy-icon {
            flex-shrink: 0;
            display: flex;
            align-items: center;
            justify-content: center;
            width: 1.5rem;
            height: 1.5rem;
            color: $placeholder;
            cursor: pointer;
            opacity: 0.3;
            transition: all 0.2s ease;
            border-radius: calc($border-radius * 0.5);
            
            &:hover {
              opacity: 1;
              color: $primary;
            }
            
            &:active {
              transform: scale(0.95);
            }
          }
          
          .log-timestamp {
            flex-shrink: 0;
            font-size: 0.75rem;
            color: $placeholder;
            min-width: 3rem;
          }
          
          .log-level {
            flex-shrink: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 0.25rem;
            font-size: 0.75rem;
            font-weight: 1000;
            min-width: 3rem;
            border-radius: $border-radius-small;
          }
          
          .log-message {
            flex: 1;
            font-size: 0.9rem;
            line-height: 1.3;
            word-break: break-all;
          }
        }
      }
    }
  }

  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    background: $card;
    font-size: 0.75rem;
    color: $placeholder;
    border-radius: $border-radius-small;

    .status-left,
    .status-right {
      display: flex;
      align-items: center;
      gap: 1rem;
    }
    
    .status-text {
      font-weight: 500;
    }
    
    .auto-scroll-indicator {
      display: flex;
      align-items: center;
      gap: 0.25rem;
      color: $green;
      font-weight: 500;
    }
  }

  // Badge variants
  .badge {
    &.danger {
      background: rgba($red, 0.1);
      color: $red;
      border: 1px solid rgba($red, 0.2);
    }
    
    &.warning {
      background: rgba($yellow, 0.1);
      color: $yellow;
      border: 1px solid rgba($yellow, 0.2);
    }
    
    &.info {
      background: rgba($blue, 0.1);
      color: $blue;
      border: 1px solid rgba($blue, 0.2);
    }
    
    &.muted {
      background: rgba($dark-300, 0.1);
      color: $dark-300;
      border: 1px solid rgba($dark-300, 0.2);
    }
    
    &.secondary {
      background: rgba($text, 0.1);
      color: $text;
      border: 1px solid rgba($text, 0.2);
    }
  }
</style>
