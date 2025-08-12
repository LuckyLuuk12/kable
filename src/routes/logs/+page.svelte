<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { 
    Icon, 
    logsService, 
    gameInstances, 
    currentLogs, 
    selectedInstanceId, 
    type GameInstance, 
    settings,
  } from '$lib';

  let selectedLogType: 'launcher' | 'game' = 'launcher';
  let logContainer: HTMLElement;
  let autoScroll = true; // Ensure auto-scroll is enabled by default
  let searchTerm = '';
  let searchMode: 'normal' | 'regex' | 'fuzzy' = 'fuzzy';
  
  // Log level filters
  $: logLevelFilters = {
    error: $settings.logging.default_log_levels.indexOf('error') !== -1,
    warn: $settings.logging.default_log_levels.indexOf('warn') !== -1,
    info: $settings.logging.default_log_levels.indexOf('info') !== -1,
    debug: $settings.logging.default_log_levels.indexOf('debug') !== -1,
  };
  
  let showLogLevelDropdown = false;
  let showCopyNotification = false;

  onMount(async () => {
    // Logs service is already initialized in the layout
    // No need to initialize again here
    
    // Check for instance parameter in URL
    const instanceParam = $page.url.searchParams.get('instance');
    if (instanceParam) {
      // Set the selected instance from the URL parameter
      selectedInstanceId.set(instanceParam);
      console.log(`Auto-selected instance from URL: ${instanceParam}`);
    }
    
    // Add event listener for clicking outside dropdown
    document.addEventListener('click', handleClickOutside);
    // Add event listener for Ctrl+A override
    document.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    // Keep logs service running since it's used globally
    // Remove event listeners
    document.removeEventListener('click', handleClickOutside);
    document.removeEventListener('keydown', handleKeyDown);
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
      case 'closed': return 'check';
      case 'crashed': return 'alert';
      case 'stopped': return 'square'; // Better icon for stopped
      default: return 'help';
    }
  }

  function getStatusColor(status: GameInstance['status']): string {
    switch (status) {
      case 'launching': return 'warning';
      case 'running': return 'success';
      case 'closed': return 'info';
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

  async function testMacroLogging() {
    try {
      // Import the invoke function from Tauri
      const { invoke } = await import('@tauri-apps/api/core');
      console.log('Triggering macro logging test...');
      const result = await invoke('test_macro_logging');
      console.log('Macro test result:', result);
      alert(`Macro test completed! Check the logs for entries with "macro_debug:" prefix.`);
    } catch (error) {
      console.error('Failed to run macro test:', error);
      alert(`Failed to run macro test: ${error}`);
    }
  }

  async function copyLogEntry(logEntry: any) {
    if (!logEntry) return;
    
    const timestamp = logEntry.timestamp instanceof Date 
      ? logEntry.timestamp 
      : new Date(logEntry.timestamp);
      
    // Clean the message for copy (remove duplicate timestamps)
    const cleanMessage = removeDuplicateTimestamp(logEntry.message || '', timestamp);
    const logText = `[${formatTime(timestamp)}] ${formatLogLevel(logEntry.level || 'info')} ${cleanMessage}`;
    try {
      await navigator.clipboard.writeText(logText);
      // Could add a toast notification here if desired
    } catch (err) {
      console.error('Failed to copy log entry:', err);
    }
  }

  // Function to detect and remove duplicate timestamps from log messages
  function removeDuplicateTimestamp(message: string, logTimestamp: Date | string): string {
    if (!message) return message;
    
    // Remove timestamp from the beginning of the message since we display it separately
    // Pattern to match timestamps like [HH:MM:SS] at the start
    const timestampPattern = /^\[\d{2}:\d{2}:\d{2}\]\s*/;
    
    let result = message.replace(timestampPattern, '');
    
    // Also remove log level if it appears right after the timestamp
    // This handles cases like "[22:54:42] INFO [Render thread/INFO]: message"
    if (result.match(/^(INFO|ERROR|WARN|DEBUG)\s+/)) {
      result = result.replace(/^(INFO|ERROR|WARN|DEBUG)\s+/, '');
    }
    
    return result.trim();
  }

  // Function to get display message (with duplicate timestamps removed)
  function getDisplayMessage(logEntry: any): string {
    if (!logEntry || !logEntry.message) return '';
    
    const timestamp = logEntry.timestamp instanceof Date 
      ? logEntry.timestamp 
      : new Date(logEntry.timestamp);
      
    return removeDuplicateTimestamp(logEntry.message, timestamp);
  }

  // Fuzzy search implementation
  function fuzzyMatch(needle: string, haystack: string): boolean {
    if (!needle || !haystack) return false;
    
    const needleLower = needle.toLowerCase();
    const haystackLower = haystack.toLowerCase();
    
    // Simple fuzzy matching: allow for missing characters and transpositions
    let needleIndex = 0;
    let score = 0;
    
    for (let i = 0; i < haystackLower.length && needleIndex < needleLower.length; i++) {
      if (haystackLower[i] === needleLower[needleIndex]) {
        needleIndex++;
        score++;
      }
    }
    
    // Calculate match ratio (how much of the needle was found)
    const matchRatio = score / needleLower.length;
    
    // Also check for substring match (higher weight)
    const substringMatch = haystackLower.includes(needleLower);
    
    // Consider it a match if:
    // 1. It's a direct substring match, OR
    // 2. At least 70% of characters are found in order
    return substringMatch || matchRatio >= 0.7;
  }

  // Advanced search function
  function matchesSearch(message: string, searchTerm: string, mode: string): boolean {
    if (!searchTerm) return true;
    if (!message) return false;

    switch (mode) {
      case 'regex':
        try {
          const regex = new RegExp(searchTerm, 'i');
          return regex.test(message);
        } catch (error) {
          // If regex is invalid, fall back to normal search
          return message.toLowerCase().includes(searchTerm.toLowerCase());
        }
      
      case 'fuzzy':
        return fuzzyMatch(searchTerm, message);
      
      case 'normal':
      default:
        return message.toLowerCase().includes(searchTerm.toLowerCase());
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

  // Handle keyboard shortcuts
  function handleKeyDown(event: KeyboardEvent) {
    // Override Ctrl+A to select all logs in the current tab
    if (event.ctrlKey && event.key === 'a') {
      // Only override if we're not in an input field
      const target = event.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
        return; // Let default behavior happen in input fields
      }

      event.preventDefault();
      selectAllCurrentLogs();
    }
  }

  // Function to select all logs in the current tab
  async function selectAllCurrentLogs() {
    if (filteredLogs.length === 0) return;

    // Show visual feedback
    showCopyNotification = true;
    
    // Generate the text for all logs
    const allLogsText = filteredLogs.map(logEntry => {
      if (!logEntry) return '';
      
      const timestamp = logEntry.timestamp instanceof Date 
        ? logEntry.timestamp 
        : new Date(logEntry.timestamp);
        
      // Clean the message for copy (remove duplicate timestamps)
      const cleanMessage = removeDuplicateTimestamp(logEntry.message || '', timestamp);
      return `[${formatTime(timestamp)}] ${formatLogLevel(logEntry.level || 'info')} ${cleanMessage}`;
    }).filter(Boolean).join('\n');

    try {
      await navigator.clipboard.writeText(allLogsText);
      console.log(`Copied ${filteredLogs.length} log entries to clipboard`);
    } catch (err) {
      console.error('Failed to copy all logs:', err);
    }
    
    // Hide notification after delay
    setTimeout(() => {
      showCopyNotification = false;
    }, 1000);
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
    // Use cleaned message for search to avoid searching duplicate timestamps
    const cleanMessage = getDisplayMessage(log);
    const matchesSearchTerm = matchesSearch(cleanMessage, searchTerm, searchMode);
    
    // Check if the log level is enabled in filters
    const logLevel = (log.level || 'info').toLowerCase();
    const matchesLevelFilter = logLevel in logLevelFilters ? 
      logLevelFilters[logLevel as keyof typeof logLevelFilters] : 
      true; // Show unknown log levels by default
    
    return matchesSearchTerm && matchesLevelFilter;
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
      <!-- Debug button for testing macro logging -->
      {#if import.meta.env.DEV}
        <button 
          class="btn btn-primary btn-sm"
          on:click={testMacroLogging}
          title="Test enhanced macro logging with Modrinth API"
        >
          <Icon name="bug" size="sm" />
          Test Macros
        </button>
      {/if}
    </div>
  </div>

  <!-- Filters Section -->
  <div class="filters-section">
    <div class="search-container">
      <Icon name="search" size="sm" />
      <input
        type="text"
        placeholder={searchMode === 'regex' ? 'Search with regex...' : 
                   searchMode === 'fuzzy' ? 'Fuzzy search (try "frge" for "forge")...' : 
                   'Search logs...'}
        bind:value={searchTerm}
        class="search-input"
      />
      <div class="search-mode-selector">
        <button
          class="search-mode-button {searchMode === 'normal' ? 'active' : ''}"
          on:click={() => searchMode = 'normal'}
          title="Normal text search"
        >
          <Icon name="text" size="sm" />
        </button>
        <button
          class="search-mode-button {searchMode === 'fuzzy' ? 'active' : ''}"
          on:click={() => searchMode = 'fuzzy'}
          title="Fuzzy search (handles typos)"
        >
          <Icon name="zap" size="sm" />
        </button>
        <button
          class="search-mode-button {searchMode === 'regex' ? 'active' : ''}"
          on:click={() => searchMode = 'regex'}
          title="Regular expression search"
        >
          <Icon name="code" size="sm" />
        </button>
      </div>
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
                <span class="log-level-count">({(activeLogEntries || []).filter(log => (log.level || 'info').toLowerCase() === level).length})</span>
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
      class="log-container {showCopyNotification ? 'copy-notification-active' : ''}"
    >
      {#if showCopyNotification}
        <div class="copy-notification">
          <div class="copy-notification-content">
            <Icon name="clipboard" size="md" />
            <span>Copied {filteredLogs.length} log entries</span>
          </div>
        </div>
      {/if}
      
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
                  <pre class="log-message-content"><code>{getDisplayMessage(logEntry)}</code></pre>
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
          {#if (filteredLogs || []).length === (currentLogsData.launcherLogs || []).length}
            Global logs: {(currentLogsData.launcherLogs || []).length} entries
          {:else}
            Global logs: {(filteredLogs || []).length} / {(currentLogsData.launcherLogs || []).length} entries
          {/if}
        {:else}
          {#if (filteredLogs || []).length === (activeLogEntries || []).length}
            {selectedLogType === 'launcher' ? 'Launcher' : 'Game'} logs: {(activeLogEntries || []).length} entries
          {:else}
            {selectedLogType === 'launcher' ? 'Launcher' : 'Game'} logs: {(filteredLogs || []).length} / {(activeLogEntries || []).length} entries
          {/if}
        {/if}
        {#if searchTerm && searchMode !== 'normal'}
          • {searchMode}
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
    user-select: none; // Disable text selection for the entire page
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
      
      .search-mode-selector {
        display: flex;
        background: $card;
        // border: 1px solid $dark-200;
        border-radius: $border-radius-tiny;
        overflow: hidden;
        
        .search-mode-button {
          display: flex;
          align-items: center;
          justify-content: center;
          padding: 0.5rem;
          background: transparent;
          border: none;
          color: $placeholder;
          cursor: pointer;
          transition: all 0.2s ease;
          min-width: 2.5rem;
          border-right: 1px solid $dark-200;
          
          &:last-child {
            border-right: none;
          }
          
          &:hover {
            background: $dark-200;
            color: $text;
          }
          
          &.active {
            background: $primary;
            color: white;
          }
          
          &:focus {
            outline: none;
            box-shadow: inset 0 0 0 2px rgba($primary, 0.3);
          }
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
            .log-level-count {
              margin-left: auto;
              color: $placeholder;
              font-size: 0.85em;
              font-weight: 500;
              letter-spacing: 0.02em;
              text-align: right;
              min-width: 2.5em;
              display: flex;
              align-items: center;
              justify-content: flex-end;
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
    
    .tab-list {
      display: flex;
      background: $container;
      border-radius: $border-radius $border-radius 0 0;
      border: 1px solid $dark-200;
      overflow-x: auto;
      gap: 0.05rem;
      
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
        border-radius: $border-radius $border-radius 0 0;

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
          &.info { background: rgba($green, 0.1); color: $green-900; }
          &.secondary { background: rgba($text, 0.2); color: $text; }
          &.muted { background: rgba($dark-300, 0.1); color: white; }
        }
      }
    }
  }

  .sub-tabs-container {
    // margin-bottom: 1rem;
    
    .sub-tab-list {
      display: flex;
      background: $container;
      border-left: 1px solid $dark-200;
      border-right: 1px solid $dark-200;
      gap: 0.05rem;
      
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
        border-radius: $border-radius $border-radius 0 0;
        
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
      border-radius: 0 0 $border-radius $border-radius;
      margin-bottom: 0.5rem;
      position: relative;
      
      &.copy-notification-active {
        .log-entries {
          .log-entry {
            background: rgba($primary, 0.1);
            border: 1px solid rgba($primary, 0.2);
          }
        }
      }
      
      .copy-notification {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba($dark-100, 0.35);
        backdrop-filter: blur(4px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        animation: fadeInOut 1s ease-in-out;
        
        .copy-notification-content {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 0.5rem;
          padding: 2rem;
          background: $card;
          border: 1px solid $primary;
          border-radius: $border-radius;
          color: $primary;
          font-weight: 600;
          box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        }
      }
      
      @keyframes fadeInOut {
        0% { opacity: 0; transform: scale(0.9); }
        20% { opacity: 1; transform: scale(1); }
        80% { opacity: 1; transform: scale(1); }
        100% { opacity: 0; transform: scale(0.9); }
      }
      
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
            min-width: 5.5rem;
            max-width: 5.5rem;
            border-radius: $border-radius-small;
          }
          
          .log-message {
            flex: 1;
            font-size: 0.9rem;
            line-height: 1.3;
            word-break: break-all;
            user-select: text; // Re-enable text selection for log content
            
            .log-message-content {
              margin: 0;
              padding: 0;
              font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', monospace;
              font-size: inherit;
              line-height: inherit;
              color: inherit;
              background: transparent;
              border: none;
              white-space: pre-wrap;
              word-wrap: break-word;
              overflow-wrap: break-word;
            }
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
