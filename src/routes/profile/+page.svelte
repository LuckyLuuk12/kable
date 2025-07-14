<script lang="ts">
  import { onMount } from 'svelte';
  import { AuthService } from '$lib/services';
  import { currentAccount, isSignedIn } from '$lib/auth';
  import Icon from '$lib/components/Icon.svelte';
  import AccountSwitcher from '$lib/components/AccountSwitcher.svelte';

  // State variables
  let isLoading = false;
  let error: string | null = null;
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

  onMount(async () => {
    await loadProfileData();
  });

  async function loadProfileData() {
    isLoading = true;
    error = null;
    
    try {
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
    <p>Manage your Microsoft account and view your Minecraft statistics</p>
  </div>

  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  <div class="profile-sections">
    <!-- Top Row: Account Switcher and Sign-in/Add Account -->
    <div class="top-row">
      <!-- Account Switcher Section -->
      <section class="profile-section account-switcher-section">
        <div class="section-header">
          <h2><Icon name="user" /> Account Switcher</h2>
        </div>
        
        <div class="account-switcher-container">
          <h3>Current Account</h3>
          <p>Switch between Microsoft accounts or add new ones from your launcher_accounts.json file.</p>
          <AccountSwitcher />
        </div>
      </section>

      <!-- Sign-in/Add Account Section -->
      <section class="profile-section sign-in-section">
        <div class="section-header">
          <h2><Icon name="user-plus" /> Account Management</h2>
        </div>
        
        {#if $isSignedIn && $currentAccount}
          <div class="account-info">
            <div class="account-avatar">
              <Icon name="user" size="xl" />
            </div>
            <div class="account-details">
              <h3>{$currentAccount.username}</h3>
              <p class="account-email">Microsoft Account</p>
              <p class="account-id">UUID: {$currentAccount.uuid}</p>
            </div>
            <div class="account-actions">
              <button on:click={signOut} class="btn btn-secondary">
                <Icon name="logout" size="sm" />
                Sign Out
              </button>
            </div>
          </div>
        {:else}
          <div class="sign-in-prompt">
            <div class="sign-in-icon">
              <Icon name="user-plus" size="xl" />
            </div>
            <h3>Sign in to Microsoft</h3>
            <p>Sign in with your Microsoft account to access online features and view your Minecraft profile.</p>
            
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
    </div>

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
    word-wrap: break-word;
    overflow-wrap: break-word;
  }

  .page-header {
    margin-bottom: 2rem;
    text-align: center;
    
    h1 {
      margin: 0 0 0.5rem;
      font-size: 2.5rem;
      font-weight: 700;
      background: linear-gradient(135deg, $primary, $tertiary);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
    
    p {
      margin: 0;
      color: $placeholder;
      font-size: 1.125rem;
      line-height: 1.6;
    }
  }

  .profile-sections {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .top-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
    align-items: stretch;
    
    @media (max-width: 1024px) {
      grid-template-columns: 1fr;
    }
  }

  .profile-section {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: $border-radius-large;
    padding: 2rem;
    transition: all 0.3s ease;
    position: relative;
    overflow: visible;
    word-wrap: break-word;
    overflow-wrap: break-word;
    
    &::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      height: 1px;
      background: linear-gradient(90deg, transparent, rgba($primary, 0.3), transparent);
    }
    
    &:hover {
      border-color: rgba($primary, 0.3);
      box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
    }

    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1.5rem;
      padding-bottom: 1rem;
      border-bottom: 1px solid rgba($dark-600, 0.5);

      h2 {
        margin: 0;
        font-size: 1.25rem;
        font-weight: 600;
        color: $text;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        position: relative;
        word-wrap: break-word;
        
        &::after {
          content: '';
          position: absolute;
          bottom: -1rem;
          left: 0;
          width: 30px;
          height: 2px;
          background: $primary;
          border-radius: 1px;
        }
      }
    }
  }
  
  .account-switcher-section {
    z-index: 20;
    position: relative;
    
    .account-switcher-container {
      padding: 1.5rem;
      background: linear-gradient(135deg, rgba($primary, 0.05) 0%, rgba($tertiary, 0.03) 100%);
      border-radius: $border-radius-large;
      border: 1px solid rgba($primary, 0.1);
      position: relative;
      overflow: visible;
      word-wrap: break-word;
      overflow-wrap: break-word;
      z-index: 10;
      
      &::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 2px;
        background: linear-gradient(90deg, $primary, $tertiary);
      }
      
      h3 {
        margin: 0 0 0.5rem;
        color: $text;
        font-size: 1.1rem;
        font-weight: 600;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        word-wrap: break-word;
        
        &::before {
          content: '👤';
          font-size: 1.2rem;
        }
      }
      
      p {
        margin: 0 0 1rem;
        color: $placeholder;
        font-size: 0.9rem;
        line-height: 1.5;
        word-wrap: break-word;
        overflow-wrap: break-word;
      }
    }
  }
  
  .sign-in-section {
    .account-info {
      display: flex;
      flex-direction: column;
      gap: 1.5rem;
      padding: 1.5rem;
      background: linear-gradient(135deg, rgba($primary, 0.05) 0%, rgba($tertiary, 0.03) 100%);
      border-radius: $border-radius;
      border: 1px solid rgba($primary, 0.1);
      word-wrap: break-word;
      overflow-wrap: break-word;

      .account-avatar {
        width: 80px;
        height: 80px;
        border-radius: 50%;
        background: linear-gradient(135deg, $primary, $tertiary);
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        flex-shrink: 0;
        box-shadow: 0 4px 15px rgba($primary, 0.3);
        position: relative;
        margin: 0 auto;
        
        &::after {
          content: '';
          position: absolute;
          inset: -2px;
          border-radius: 50%;
          background: linear-gradient(135deg, $primary, $tertiary);
          z-index: -1;
          opacity: 0.5;
          filter: blur(8px);
        }
      }

      .account-details {
        text-align: center;
        word-wrap: break-word;
        overflow-wrap: break-word;

        h3 {
          margin: 0 0 0.5rem;
          font-size: 1.5rem;
          font-weight: 600;
          color: $text;
          background: linear-gradient(135deg, $primary, $tertiary);
          -webkit-background-clip: text;
          -webkit-text-fill-color: transparent;
          background-clip: text;
          word-wrap: break-word;
        }

        .account-email {
          margin: 0 0 0.25rem;
          color: $text;
          font-size: 1rem;
          font-weight: 500;
          word-wrap: break-word;
        }

        .account-id {
          margin: 0;
          color: $placeholder;
          font-size: 0.875rem;
          font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
          background: rgba($dark-600, 0.3);
          padding: 0.5rem;
          border-radius: 4px;
          word-wrap: break-word;
          overflow-wrap: break-word;
          line-height: 1.4;
        }
      }

      .account-actions {
        display: flex;
        justify-content: center;
      }
    }

    .sign-in-prompt {
      text-align: center;
      padding: 2rem;
      background: linear-gradient(135deg, rgba($primary, 0.03) 0%, rgba($tertiary, 0.02) 100%);
      border-radius: $border-radius;
      border: 1px dashed rgba($primary, 0.2);
      word-wrap: break-word;
      overflow-wrap: break-word;

      .sign-in-icon {
        margin-bottom: 1.5rem;
        color: $primary;
        opacity: 0.8;
      }

      h3 {
        margin: 0 0 1rem;
        font-size: 1.25rem;
        font-weight: 600;
        color: $text;
        word-wrap: break-word;
      }

      p {
        margin: 0 0 2rem;
        color: $placeholder;
        line-height: 1.6;
        word-wrap: break-word;
        overflow-wrap: break-word;
      }
    }
  }

  .stats-section {
    grid-column: 1 / -1; /* Full width in the main container */
    
    .stats-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 1.5rem;

      .stat-card {
        background: linear-gradient(135deg, rgba($primary, 0.03) 0%, rgba($tertiary, 0.02) 100%);
        border: 1px solid rgba($dark-600, 0.6);
        border-radius: $border-radius;
        padding: 1.5rem;
        display: flex;
        align-items: center;
        gap: 1rem;
        transition: all 0.3s ease;
        position: relative;
        overflow: hidden;
        word-wrap: break-word;
        overflow-wrap: break-word;
        
        &::before {
          content: '';
          position: absolute;
          top: 0;
          left: 0;
          width: 100%;
          height: 2px;
          background: linear-gradient(90deg, $primary, $tertiary);
          transform: translateX(-100%);
          transition: transform 0.3s ease;
        }
        
        &:hover {
          border-color: rgba($primary, 0.3);
          transform: translateY(-2px);
          box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
          
          &::before {
            transform: translateX(0);
          }
        }

        .stat-icon {
          width: 48px;
          height: 48px;
          border-radius: $border-radius;
          background: linear-gradient(135deg, rgba($primary, 0.15), rgba($tertiary, 0.1));
          display: flex;
          align-items: center;
          justify-content: center;
          color: $primary;
          flex-shrink: 0;
          position: relative;
          
          &::after {
            content: '';
            position: absolute;
            inset: -1px;
            border-radius: $border-radius;
            background: linear-gradient(135deg, $primary, $tertiary);
            z-index: -1;
            opacity: 0.3;
            filter: blur(4px);
          }
        }

        .stat-content {
          flex: 1;
          word-wrap: break-word;
          overflow-wrap: break-word;
          
          h4 {
            margin: 0 0 0.25rem;
            font-size: 0.875rem;
            font-weight: 500;
            color: $placeholder;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            word-wrap: break-word;
          }

          .stat-value {
            margin: 0;
            font-size: 1.25rem;
            font-weight: 600;
            color: $text;
            background: linear-gradient(135deg, $primary, $tertiary);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            word-wrap: break-word;
            overflow-wrap: break-word;
            line-height: 1.3;
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

  @media (max-width: 768px) {
    .profile-section {
      padding: 1rem;
    }

    .account-info {
      flex-direction: column;
      text-align: center;
      gap: 1rem !important;
    }

    .stats-grid {
      grid-template-columns: 1fr !important;
    }
    
    .profile-sections {
      grid-template-columns: 1fr !important;
    }
    
    .sign-in-prompt {
      padding: 2rem 1rem !important;
    }
  }
</style>
