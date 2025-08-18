<script lang="ts">
  import { onMount } from 'svelte';
import { Icon, installations, isLoadingInstallations, installationsError, type KableInstallation, Launcher, InstallationService } from '$lib';
    import InstallationsList from '$lib/components/installations/InstallationsList.svelte';


  // State variables
  let lastPlayedInstallations: KableInstallation[] = [];
  let error: string | null = null;
  let viewMode: 'grid' | 'list' = 'grid';
  let isLaunching = false;
  let launchStatus = '';
  let openDropdownId: string | null = null;
  
  // RAM allocation state
  let ramAllocation = 2048; // Default 2GB in MB
  let ramInputValue = '2048'; // String value for text input

  // Subscribe to the installations store and update RAM allocation
  $: {
    console.log('Total installations:', $installations.length);
    
    lastPlayedInstallations = $installations
      .sort((a: KableInstallation, b: KableInstallation) => {
        const aTime = new Date(a.last_used || 0).getTime();
        const bTime = new Date(b.last_used || 0).getTime();
        return bTime - aTime;
      })
      .slice(0, 8); // Show up to 8 installations
      
    console.log('Last played installations:', lastPlayedInstallations.length);

    // Update RAM allocation when installation changes
    if (lastPlayedInstallations.length > 0) {
      const latestInstallation = lastPlayedInstallations[0];
      console.log('Latest installation java_args:', latestInstallation.java_args);
      
      // Extract RAM from java_args (look for -Xmx)
      const xmxArg = latestInstallation.java_args?.find(arg => arg.startsWith('-Xmx'));
      if (xmxArg) {
        console.log('Found Xmx arg:', xmxArg);
        const memValue = xmxArg.replace('-Xmx', '').toLowerCase();
        if (memValue.endsWith('g')) {
          ramAllocation = parseInt(memValue) * 1024;
        } else if (memValue.endsWith('m')) {
          ramAllocation = parseInt(memValue);
        }
        ramInputValue = ramAllocation.toString();
        console.log('Set RAM allocation to:', ramAllocation, 'MB');
      }
    }
  }

  // Subscribe to loading and error states
  $: isLoading = $isLoadingInstallations;
  $: if ($installationsError) {
    error = $installationsError;
  }

  // Initialize on component mount
  onMount(() => {
    console.log('Home page mounted');
    // GameManager is already initialized by the layout with installations loaded

    // Add click outside handler for dropdown
    function handleClickOutside(event: MouseEvent) {
      const target = event.target as Element;
      if (!target.closest('.dropdown-container')) {
        openDropdownId = null;
      }
    }
    document.addEventListener('click', handleClickOutside);
    
    // Return cleanup function
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  });

  function toggleViewMode() {
    viewMode = viewMode === 'grid' ? 'list' : 'grid';
  }

  function getModLoaderIcon(modLoader: string) {
    switch (modLoader) {
      case 'fabric': return 'fabric';
      case 'forge': return 'hammer';
      default: return 'cube';
    }
  }

  function toggleDropdown(installationId: string) {
    openDropdownId = openDropdownId === installationId ? null : installationId;
  }

  function closeDropdown() {
    openDropdownId = null;
  }

  async function handlePlay() {
    isLaunching = true;
    launchStatus = 'Preparing to launch...';
    let result;
    
    try {      
      // Try to launch the most recent installation
      if (lastPlayedInstallations.length > 0) {
        console.log('Launching installation:', lastPlayedInstallations[0]);
        launchStatus = `Launching ${lastPlayedInstallations[0].name}...`;
        // Launch the installation directly using Launcher
        result = await Launcher.launchInstallation(lastPlayedInstallations[0]);
      } else {
        launchStatus = 'Launching default Minecraft...';
        // Use Launcher for quick launch fallback
        result = await Launcher.launchLatest();
      }
      
      if (result.success) {         
        launchStatus = 'Launched Minecraft!';
        // Refresh installations to update last played
        setTimeout(() => {
          InstallationService.loadInstallations();
        }, 1000);
      } else {
        launchStatus = `Launch failed: ${result.error || 'Unknown error'}`;
      }
      
    } catch (err) {
      console.error('Launch error:', err);
      launchStatus = `Launch failed: ${err}`;
    } finally {
      // Reset the button state quickly since Minecraft is now running independently
      setTimeout(() => {
        launchStatus = '';
        isLaunching = false;
      }, result?.success ? 2000 : 5000);
    }
  }

  async function handleInstallationLaunch(installation: KableInstallation) {
    const launchButton = event?.target as HTMLButtonElement;
    const originalText = launchButton?.textContent || '';
    
    if (launchButton) {
      launchButton.disabled = true;
      launchButton.textContent = 'Launching...';
    }
    
    try {
      // Launch the installation directly using Launcher
      const result = await Launcher.launchInstallation(installation);
      
      if (result.success) {
        if (launchButton) {
          launchButton.textContent = 'Launched!';
        }
        // Refresh installations to update last played
        setTimeout(() => {
          InstallationService.loadInstallations();
        }, 1000);
      } else {
        alert(`Launch failed: ${result.error || 'Unknown error'}`);
      }
    } catch (err) {
      console.error('Installation launch error:', err);
      alert(`Launch failed: ${err}`);
    } finally {
      // Reset button state after a short delay
      setTimeout(() => {
        if (launchButton) {
          launchButton.disabled = false;
          launchButton.textContent = originalText;
        }
      }, 2000);
    }
  }

  // RAM allocation functions
  function updateRamFromSlider() {
    ramInputValue = ramAllocation.toString();
  }

  function updateRamFromInput() {
    const value = parseInt(ramInputValue);
    if (!isNaN(value) && value >= 512 && value <= 32768) {
      // Round to nearest 256MB increment
      const roundedValue = Math.round(value / 256) * 256;
      // Ensure it stays within bounds after rounding
      const clampedValue = Math.max(512, Math.min(32768, roundedValue));
      ramAllocation = clampedValue;
      ramInputValue = clampedValue.toString();
    } else {
      // Reset to current valid value if invalid input
      ramInputValue = ramAllocation.toString();
    }
  }

  function formatRamDisplay(mb: number): string {
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(1)}GB`;
    }
    return `${mb}MB`;
  }
</script>

<div class="page-wrapper">

  <!-- Installations List Section -->
  <div class="installations-section">
    <InstallationsList isGrid isSmall limit={15}/>
  </div>

  <!-- Bottom Controls Section -->
  <div class="bottom-controls">
    <!-- Play Button (Centered) -->
    <div class="play-section">
      <button class="play-button" on:click={handlePlay} disabled={isLaunching || lastPlayedInstallations.length === 0}>
        <Icon name={isLaunching ? "refresh" : "play"} size="md" forceType="svg" />
        <span>{isLaunching ? 'Launching...' : 'Play Minecraft'}</span>
      </button>
      {#if lastPlayedInstallations.length === 0}
        <p class="no-installations">No installations found. Please check your Minecraft directory in settings.</p>
      {/if}
      {#if launchStatus}
        <p class="launch-status" class:error={launchStatus.includes('fail') || launchStatus.includes('error')}>
          {launchStatus}
        </p>
      {/if}
    </div>

    <!-- RAM Allocation Controls (Bottom Right) -->
    <div class="ram-controls">
      <div class="ram-header">
        <span class="installation-name">
          {lastPlayedInstallations.length > 0 ? lastPlayedInstallations[0].name : 'No Installation'}
        </span>
        <span class="ram-display">{formatRamDisplay(ramAllocation)}</span>
      </div>
      
      <div class="ram-inputs">
        <!-- Slider Input -->
        <div class="ram-slider-container">
          <input
            type="range"
            class="ram-slider"
            bind:value={ramAllocation}
            on:input={updateRamFromSlider}
            min="512"
            max="32768"
            step="256"
          />
          <div class="slider-labels">
            <span>512MB</span>
            <span>8GB</span>
            <span>16GB</span>
            <span>32GB</span>
          </div>
        </div>
        
        <!-- Text Input -->
        <div class="ram-text-container">
          <input
            type="text"
            class="ram-input"
            bind:value={ramInputValue}
            on:blur={updateRamFromInput}
            on:keydown={(e) => e.key === 'Enter' && updateRamFromInput()}
            placeholder="2048"
          />
          <span class="ram-unit">MB</span>
        </div>
      </div>
    </div>
  </div>
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;
  .page-wrapper {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: $container;
    overflow: hidden;
    border-radius: $border-radius;
  }

  .installations-section {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 2rem;
    margin-bottom: -2rem;
  }

  .bottom-controls {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    padding: 1.5rem 2rem;
    background: linear-gradient(to top, $container 40%, rgba($container, 0.6) 80%, rgba($container, 0.15) 90%, transparent 100%);
    backdrop-filter: blur(0.125rem);
    flex-shrink: 0;
    position: relative;
    z-index: 10;
    min-height: 5rem;
  }

  .play-section {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
  }

  .ram-controls {
    position: absolute;
    bottom: 0;
    right: 0;
    margin: 0;
    padding: 1rem;
    background: $card;
    border: 1px solid $dark-600;
    border-radius: 0.5rem;
    max-width: 23.75rem;
    min-width: 20rem;

    .ram-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 0.75rem;
      font-size: 0.875rem;
      font-weight: 500;
      color: $text;

      .installation-name {
        color: $text;
        font-weight: 500;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        flex: 1;
        margin-right: 0.5rem;
      }

      .ram-display {
        color: $primary;
        font-weight: 600;
        flex-shrink: 0;
      }
    }

    .ram-inputs {
      display: flex;
      gap: 0.75rem;
      align-items: center;

      .ram-slider-container {
        flex: 1;
        
        .ram-slider {
          width: 100%;
          height: 0.25rem;
          border-radius: 0.125rem;
          outline: none;
          appearance: none;
          cursor: pointer;
          
          &::-webkit-slider-thumb {
            appearance: none;
            width: 1rem;
            height: 1rem;
            background: $primary;
            border-radius: 50%;
            cursor: pointer;
            transition: all 0.2s ease;
            
            &:hover {
              background: $primary-600;
              transform: scale(1.1);
            }
          }
          
          &::-moz-range-thumb {
            width: 1rem;
            height: 1rem;
            background: $primary;
            border-radius: 50%;
            border: none;
            cursor: pointer;
            transition: all 0.2s ease;
            
            &:hover {
              background: $primary-600;
              transform: scale(1.1);
            }
          }
        }

        .slider-labels {
          display: flex;
          justify-content: space-between;
          margin-top: 0.25rem;
          font-size: 0.625rem;
          color: $placeholder;
        }
      }

      .ram-text-container {
        display: flex;
        align-items: center;
        gap: 0.375rem;
        flex-shrink: 0;

        .ram-input {
          width: 3.75rem;
          padding: 0.375rem 0.5rem;
          background: $dark-600;
          border: 1px solid $dark-500;
          border-radius: 0.25rem;
          color: $text;
          font-size: 0.75rem;
          text-align: center;
          transition: border-color 0.2s ease;

          &:focus {
            outline: none;
            border-color: $primary;
          }

          &::placeholder {
            color: $placeholder;
          }
        }

        .ram-unit {
          font-size: 0.75rem;
          color: $placeholder;
          font-weight: 500;
        }
      }
    }
  }

  .play-button {
    display: inline-flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 2rem;
    background: $primary;
    color: white;
    border: none;
    border-radius: 0.75rem;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    width: 20rem;
    justify-content: center;

    &:hover:not(:disabled) {
      background: $primary-600;
      transform: translateY(-0.125rem);
    }

    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
      transform: none;
    }
  }

  .no-installations {
    margin: 1rem 0 0;
    color: $placeholder;
    font-size: 0.875rem;
  }
  
  .launch-status {
    margin: 1rem 0 0;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    font-size: 0.875rem;
    background: rgba($green, 0.1);
    color: $green;
    border: 1px solid rgba($green, 0.3);
    
    &.error {
      background: rgba($red, 0.1);
      color: $red;
      border-color: rgba($red, 0.3);
    }
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @keyframes dropdownSlide {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  // Responsive design
  @media (max-width: 768px) {
    .play-section {
      padding-left: 1rem;
      padding-right: 1rem;
    }

    .play-button {
      min-width: auto;
      width: 100%;
      max-width: 300px;
    }
  }
</style>
