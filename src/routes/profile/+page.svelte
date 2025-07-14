<script lang="ts">
  import { onMount } from 'svelte';
  import { AuthService } from '$lib/services';
  import { currentAccount, isSignedIn } from '$lib/auth';
  import Icon from '$lib/components/Icon.svelte';
  import type { MinecraftSkin } from '$lib/types';

  // State variables
  let isLoading = false;
  let error: string | null = null;
  let skins: MinecraftSkin[] = [];
  let selectedSkinIndex = 0;
  let userStats = {
    totalPlaytime: 0,
    lastPlayed: null as string | null,
    favoriteDimension: 'Overworld',
    worldsCreated: 0
  };

  // Device Code Flow state
  let deviceCodeMessage = '';
  let userCode = '';
  let verificationUrl = '';
  let isPolling = false;
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  // Mock skins data for now
  const mockSkins: MinecraftSkin[] = [
    {
      id: 'steve',
      name: 'Steve (Default)',
      file_path: '',
      file_name: 'steve.png',
      is_slim: false,
      preview_url: undefined,
      source: 'Default' as any,
      created_date: 0,
      last_used: undefined
    },
    {
      id: 'alex',
      name: 'Alex (Default)',
      file_path: '',
      file_name: 'alex.png',
      is_slim: true,
      preview_url: undefined,
      source: 'Default' as any,
      created_date: 0,
      last_used: undefined
    }
  ];

  onMount(async () => {
    await loadProfileData();
  });

  async function loadProfileData() {
    isLoading = true;
    error = null;
    
    try {
      // TODO: Load skins from backend
      skins = mockSkins;
      
      // TODO: Load user stats from Minecraft data
      userStats = {
        totalPlaytime: 247, // hours
        lastPlayed: '2025-01-13T10:30:00Z',
        favoriteDimension: 'The Nether',
        worldsCreated: 12
      };
      
    } catch (err) {
      console.error('Failed to load profile data:', err);
      error = `Failed to load profile data: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function signIn() {
    try {
      isLoading = true;
      const account = await AuthService.authenticateWithMicrosoft();
      currentAccount.set(account);
      console.log('Successfully authenticated:', account);
    } catch (err) {
      console.error('Sign in failed:', err);
      error = `Sign in failed: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function signInWithDeviceCode() {
    try {
      isLoading = true;
      error = null;
      
      // Start device code flow
      const response = await AuthService.startDeviceCodeAuth();
      const [code, url] = response.split('|');
      
      userCode = code;
      verificationUrl = url;
      deviceCodeMessage = `Code: ${code}`;
      
      // Copy code to clipboard automatically
      try {
        await AuthService.copyToClipboard(code);
        deviceCodeMessage = `Code: ${code} (copied to clipboard!)`;
      } catch (err) {
        console.warn('Failed to copy to clipboard:', err);
      }
      
      // Start polling for completion
      isPolling = true;
      pollInterval = setInterval(async () => {
        try {
          const account = await AuthService.pollDeviceCodeAuth();
          if (account) {
            // Authentication successful
            clearInterval(pollInterval!);
            pollInterval = null;
            isPolling = false;
            deviceCodeMessage = '';
            userCode = '';
            verificationUrl = '';
            currentAccount.set(account);
            console.log('Successfully authenticated with device code:', account);
          }
        } catch (err) {
          // Authentication failed or expired
          clearInterval(pollInterval!);
          pollInterval = null;
          isPolling = false;
          deviceCodeMessage = '';
          userCode = '';
          verificationUrl = '';
          error = `Device code authentication failed: ${err}`;
        }
      }, 3000); // Poll every 3 seconds
      
    } catch (err) {
      console.error('Device code sign in failed:', err);
      error = `Device code sign in failed: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  function cancelDeviceCodeAuth() {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
    isPolling = false;
    deviceCodeMessage = '';
    userCode = '';
    verificationUrl = '';
    isLoading = false;
  }

  async function copyCodeAgain() {
    if (userCode) {
      try {
        await AuthService.copyToClipboard(userCode);
        deviceCodeMessage = `Code: ${userCode} (copied to clipboard!)`;
        setTimeout(() => {
          if (userCode) {
            deviceCodeMessage = `Code: ${userCode}`;
          }
        }, 2000);
      } catch (err) {
        console.warn('Failed to copy to clipboard:', err);
      }
    }
  }

  async function signOut() {
    try {
      currentAccount.set(null);
      // Clear localStorage if needed
      localStorage.removeItem('kable_account');
    } catch (err) {
      console.error('Sign out failed:', err);
      error = `Sign out failed: ${err}`;
    }
  }

  async function changeSkin(skinIndex: number) {
    selectedSkinIndex = skinIndex;
    // TODO: Apply skin change
    console.log('Changing to skin:', skins[skinIndex].name);
  }

  async function uploadSkin() {
    // TODO: Implement skin upload
    alert('Skin upload feature coming soon!');
  }

  function formatPlaytime(hours: number): string {
    if (hours < 24) {
      return `${hours} hours`;
    }
    const days = Math.floor(hours / 24);
    const remainingHours = hours % 24;
    return `${days} days, ${remainingHours} hours`;
  }
</script>

<div class="profile-page">
  <div class="page-header">
    <h1>Profile & Account</h1>
    <p>Manage your Microsoft account, skins, and view your Minecraft statistics</p>
  </div>

  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  <div class="profile-sections">
    <!-- Microsoft Account Section -->
    <section class="profile-section account-section">
      <div class="section-header">
        <h2><Icon name="user" /> Microsoft Account</h2>
      </div>
      
      {#if $isSignedIn && $currentAccount}
        <div class="account-info">
          <div class="account-avatar">
            <Icon name="user" size="xl" />
          </div>
          <div class="account-details">
            <h3>{$currentAccount.username}</h3>
            <p class="account-email">Minecraft Account</p>
            <p class="account-id">UUID: {$currentAccount.uuid}</p>
          </div>
          <button on:click={signOut} class="btn btn-secondary">
            <Icon name="logout" size="sm" />
            Sign Out
          </button>
        </div>
      {:else}
        <div class="sign-in-prompt">
          <div class="sign-in-icon">
            <Icon name="user-plus" size="xl" />
          </div>
          <h3>Sign in to Microsoft</h3>
          <p>Sign in with your Microsoft account to access online features, sync your skins, and view your Minecraft profile.</p>
          
          {#if deviceCodeMessage}
            <div class="device-code-info">
              <div class="device-code-header">
                <Icon name="info" size="sm" />
                <span>Authentication Started</span>
              </div>
              
              <div class="device-code-instructions">
                <p>A browser window has opened to:</p>
                <div class="url-display">
                  <code>{verificationUrl}</code>
                </div>
                
                <p>Enter this code when prompted:</p>
                <div class="code-display">
                  <code class="user-code">{userCode}</code>
                  <button on:click={copyCodeAgain} class="copy-btn" title="Copy code again">
                    <Icon name="duplicate" size="sm" />
                  </button>
                </div>
                
                <div class="code-status">
                  {deviceCodeMessage}
                </div>
              </div>
              
              {#if isPolling}
                <div class="polling-status">
                  <Icon name="refresh" size="sm" />
                  <span>Waiting for you to complete authentication...</span>
                </div>
              {/if}
              
              <div class="device-code-actions">
                <button on:click={cancelDeviceCodeAuth} class="btn btn-secondary">
                  Cancel
                </button>
              </div>
            </div>
          {:else}
            <div class="auth-options">
              <button on:click={signIn} class="btn btn-primary" disabled={isLoading}>
                <Icon name="microsoft" size="sm" />
                {isLoading ? 'Signing in...' : 'Sign in with Microsoft'}
              </button>
              
              <div class="auth-separator">
                <span>or</span>
              </div>
              
              <button on:click={signInWithDeviceCode} class="btn btn-secondary" disabled={isLoading}>
                <Icon name="qr-code" size="sm" />
                Use Device Code (Alternative)
              </button>
              
              <p class="auth-help">
                <Icon name="info" size="sm" />
                If the standard sign-in doesn't work, try the device code option.
              </p>
            </div>
          {/if}
        </div>
      {/if}
    </section>

    <!-- Skins & Appearance Section -->
    <section class="profile-section skins-section">
      <div class="section-header">
        <h2><Icon name="palette" /> Skins & Appearance</h2>
        <button on:click={uploadSkin} class="btn btn-secondary">
          <Icon name="upload" size="sm" />
          Upload Skin
        </button>
      </div>
      
      {#if isLoading}
        <div class="loading-state">
          <Icon name="refresh" size="md" />
          <span>Loading skins...</span>
        </div>
      {:else}
        <div class="skins-grid">
          {#each skins as skin, index}
            <div 
              class="skin-card" 
              class:selected={index === selectedSkinIndex}
              on:click={() => changeSkin(index)}
              on:keydown={(e) => e.key === 'Enter' && changeSkin(index)}
              role="button"
              tabindex="0"
            >
              <div class="skin-preview">
                <div class="skin-placeholder">
                  <Icon name="user" size="lg" />
                </div>
                <div class="skin-model-type">
                  {skin.is_slim ? 'Slim' : 'Classic'}
                </div>
              </div>
              <div class="skin-info">
                <h4>{skin.name}</h4>
                {#if index === selectedSkinIndex}
                  <span class="current-skin">Current</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Statistics Section -->
    <section class="profile-section stats-section">
      <div class="section-header">
        <h2><Icon name="chart" /> Minecraft Statistics</h2>
      </div>
      
      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="clock" size="md" />
          </div>
          <div class="stat-content">
            <h4>Total Playtime</h4>
            <p class="stat-value">{formatPlaytime(userStats.totalPlaytime)}</p>
          </div>
        </div>
        
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="calendar" size="md" />
          </div>
          <div class="stat-content">
            <h4>Last Played</h4>
            <p class="stat-value">
              {userStats.lastPlayed ? new Date(userStats.lastPlayed).toLocaleDateString() : 'Never'}
            </p>
          </div>
        </div>
        
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="world" size="md" />
          </div>
          <div class="stat-content">
            <h4>Favorite Dimension</h4>
            <p class="stat-value">{userStats.favoriteDimension}</p>
          </div>
        </div>
        
        <div class="stat-card">
          <div class="stat-icon">
            <Icon name="folder" size="md" />
          </div>
          <div class="stat-content">
            <h4>Worlds Created</h4>
            <p class="stat-value">{userStats.worldsCreated}</p>
          </div>
        </div>
      </div>
    </section>
  </div>
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;

  .profile-page {
    max-width: 1200px;
    margin: 0 auto;
  }

  .profile-sections {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .profile-section {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: $border-radius;
    padding: 2rem;

    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1.5rem;

      h2 {
        margin: 0;
        font-size: 1.25rem;
        font-weight: 600;
        color: $text;
        display: flex;
        align-items: center;
        gap: 0.5rem;
      }
    }
  }

  .account-section {
    .account-info {
      display: flex;
      align-items: center;
      gap: 1.5rem;

      .account-avatar {
        width: 80px;
        height: 80px;
        border-radius: 50%;
        background: $primary;
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        flex-shrink: 0;
      }

      .account-details {
        flex: 1;

        h3 {
          margin: 0 0 0.5rem;
          font-size: 1.5rem;
          font-weight: 600;
          color: $text;
        }

        .account-email {
          margin: 0 0 0.25rem;
          color: $text;
          font-size: 1rem;
        }

        .account-id {
          margin: 0;
          color: $placeholder;
          font-size: 0.875rem;
          font-family: monospace;
        }
      }
    }

    .sign-in-prompt {
      text-align: center;
      padding: 2rem;

      .sign-in-icon {
        margin-bottom: 1rem;
        color: $placeholder;
      }

      h3 {
        margin: 0 0 1rem;
        font-size: 1.25rem;
        font-weight: 600;
        color: $text;
      }

      p {
        margin: 0 0 2rem;
        color: $placeholder;
        line-height: 1.5;
        max-width: 400px;
        margin-left: auto;
        margin-right: auto;
      }
    }
  }

  .skins-section {
    .skins-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
      gap: 1rem;

      .skin-card {
        background: $background;
        border: 2px solid $dark-600;
        border-radius: $border-radius;
        padding: 1rem;
        text-align: center;
        cursor: pointer;
        transition: all 0.2s ease;

        &:hover {
          border-color: $primary;
          transform: translateY(-2px);
        }

        &.selected {
          border-color: $primary;
          background: rgba($primary, 0.1);
        }

        .skin-preview {
          position: relative;
          margin-bottom: 1rem;

          .skin-placeholder {
            width: 80px;
            height: 80px;
            margin: 0 auto 0.5rem;
            background: $container;
            border-radius: $border-radius;
            display: flex;
            align-items: center;
            justify-content: center;
            color: $placeholder;
          }

          .skin-model-type {
            font-size: 0.75rem;
            color: $placeholder;
            text-transform: uppercase;
            font-weight: 500;
          }
        }

        .skin-info {
          h4 {
            margin: 0 0 0.5rem;
            font-size: 0.875rem;
            font-weight: 600;
            color: $text;
          }

          .current-skin {
            font-size: 0.75rem;
            color: $primary;
            font-weight: 600;
            text-transform: uppercase;
          }
        }
      }
    }
  }

  .stats-section {
    .stats-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
      gap: 1rem;

      .stat-card {
        background: $background;
        border: 1px solid $dark-600;
        border-radius: $border-radius;
        padding: 1.5rem;
        display: flex;
        align-items: center;
        gap: 1rem;

        .stat-icon {
          width: 48px;
          height: 48px;
          border-radius: $border-radius;
          background: rgba($primary, 0.1);
          display: flex;
          align-items: center;
          justify-content: center;
          color: $primary;
          flex-shrink: 0;
        }

        .stat-content {
          h4 {
            margin: 0 0 0.25rem;
            font-size: 0.875rem;
            font-weight: 500;
            color: $placeholder;
            text-transform: uppercase;
          }

          .stat-value {
            margin: 0;
            font-size: 1.25rem;
            font-weight: 600;
            color: $text;
          }
        }
      }
    }
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    background: rgba($red, 0.1);
    border: 1px solid $red;
    border-radius: $border-radius;
    color: $red;
    margin-bottom: 1rem;
  }

  // Device Code Authentication Styles
  .auth-options {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    align-items: center;
    max-width: 300px;
    margin: 0 auto;
  }

  .auth-separator {
    position: relative;
    width: 100%;
    text-align: center;
    
    span {
      background: $container;
      padding: 0 1rem;
      color: $placeholder;
      font-size: 0.875rem;
    }
    
    &::before {
      content: '';
      position: absolute;
      top: 50%;
      left: 0;
      right: 0;
      height: 1px;
      background: $dark-600;
      z-index: -1;
    }
  }

  .device-code-info {
    padding: 1.5rem;
    background: rgba($primary, 0.05);
    border: 1px solid rgba($primary, 0.1);
    border-radius: $border-radius;
    text-align: center;
    
    .device-code-header {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      margin-bottom: 1rem;
      color: $primary;
      font-weight: 600;
    }
    
    .device-code-instructions {
      margin-bottom: 1rem;
      
      p {
        margin: 0 0 0.5rem;
        color: $text;
        font-size: 0.9rem;
      }
      
      .url-display {
        margin-bottom: 1rem;
        padding: 0.75rem;
        background: $background;
        border: 1px solid $dark-600;
        border-radius: $border-radius;
        
        code {
          color: $text;
          font-size: 0.875rem;
          word-break: break-all;
        }
      }
      
      .code-display {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        margin-bottom: 1rem;
        padding: 1rem;
        background: $container;
        border: 2px solid $primary;
        border-radius: $border-radius;
        
        .user-code {
          font-size: 1.25rem;
          font-weight: 700;
          color: $primary;
          letter-spacing: 0.1em;
        }
        
        .copy-btn {
          background: none;
          border: 1px solid rgba($primary, 0.3);
          border-radius: 4px;
          padding: 0.25rem;
          color: $primary;
          cursor: pointer;
          transition: all 0.2s ease;
          
          &:hover {
            background: rgba($primary, 0.1);
            border-color: $primary;
          }
        }
      }
      
      .code-status {
        font-size: 0.875rem;
        color: $green;
        font-weight: 500;
      }
    }
    
    .polling-status {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      margin-bottom: 1rem;
      color: $placeholder;
      
      :global(.icon) {
        animation: spin 1s linear infinite;
      }
    }
    
    .device-code-actions {
      margin-top: 1rem;
    }
  }

  .auth-help {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: $placeholder;
    margin: 0;
    text-align: center;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 2rem;
    color: $placeholder;
  }

  @media (max-width: 768px) {
    .profile-section {
      padding: 1rem;
    }

    .account-info {
      flex-direction: column;
      text-align: center;
    }

    .skins-grid {
      grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    }

    .stats-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
