<script lang="ts">
  import { AuthManager, GameManager } from '$lib';

  // Subscribe to reactive stores
  $: currentAccount = AuthManager.currentAccount;
  $: installations = GameManager.installations;

  // Methods
  async function signInWithMicrosoft() {
    try {
      await AuthManager.signInWithMicrosoft();
    } catch (error) {
      console.error('Sign in failed:', error);
    }
  }

  async function signOut() {
    try {
      await AuthManager.signOut();
    } catch (error) {
      console.error('Sign out failed:', error);
    }
  }

  async function refreshProfiles() {
    try {
      await GameManager.refreshInstallations();
    } catch (error) {
      console.error('Failed to refresh profiles:', error);
    }
  }

  async function launchGame(installationId: string) {
    try {
      await GameManager.launchGame(installationId);
    } catch (error) {
      console.error('Failed to launch game:', error);
    }
  }
</script>

<div class="home-page">
  <div class="page-header">
    <h1>Game Launcher</h1>
    <p>Launch your Minecraft installations</p>
  </div>

  <!-- Quick Auth Section -->
  <section class="quick-auth">
    {#if $currentAccount}
      <div class="user-info">
        <img src={$currentAccount.avatar_url} alt="Avatar" class="avatar" />
        <div class="user-details">
          <span class="username">{$currentAccount.username}</span>
          <span class="status">Ready to play</span>
        </div>
        <button on:click={signOut} class="sign-out-btn">Sign Out</button>
      </div>
    {:else}
      <div class="sign-in-prompt">
        <div class="prompt-content">
          <h3>Sign in to play Minecraft</h3>
          <p>Connect your Microsoft account to access your games</p>
          <button on:click={signInWithMicrosoft} class="sign-in-btn">
            Sign in with Microsoft
          </button>
        </div>
      </div>
    {/if}
  </section>

  <!-- Game Installations Section -->
  <section class="game-launcher">
    <div class="section-header">
      <h2>Your Installations</h2>
      <button on:click={refreshProfiles} class="refresh-btn">
        🔄 Refresh
      </button>
    </div>
    
    {#if $installations && $installations.length > 0}
      <div class="installations-grid">
        {#each $installations as installation}
          <div class="installation-card">
            <div class="installation-header">
              <h3>{installation.name}</h3>
              <span class="version-badge">{installation.version}</span>
            </div>
            
            <div class="installation-details">
              <div class="detail-row">
                <span class="label">Type:</span>
                <span class="value">{installation.installation_type}</span>
              </div>
              {#if installation.mod_loader}
                <div class="detail-row">
                  <span class="label">Mod Loader:</span>
                  <span class="value">{installation.mod_loader}</span>
                </div>
              {/if}
            </div>
            
            <div class="installation-actions">
              <button 
                on:click={() => launchGame(installation.id)} 
                class="launch-btn"
                disabled={!$currentAccount}
              >
                {#if !$currentAccount}
                  🔒 Sign in to play
                {:else}
                  ▶️ Launch
                {/if}
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="no-installations">
        <div class="empty-state">
          <div class="empty-icon">📦</div>
          <h3>No installations found</h3>
          <p>Click refresh to scan for existing Minecraft installations, or create a new one from the profile page.</p>
          <button on:click={refreshProfiles} class="refresh-btn primary">
            Scan for installations
          </button>
        </div>
      </div>
    {/if}
  </section>
</div>

<style lang="scss">
  .home-page {
    max-width: 1200px;
    margin: 0 auto;
  }

  .page-header {
    text-align: center;
    margin-bottom: 2rem;
    
    h1 {
      margin: 0 0 0.5rem 0;
      font-size: 2.5rem;
      font-weight: 700;
      background: linear-gradient(135deg, var(--primary), var(--accent));
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
    
    p {
      margin: 0;
      color: var(--text-muted);
      font-size: 1.1rem;
    }
  }

  .quick-auth {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 1rem;
    padding: 1.5rem;
    margin-bottom: 2rem;
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: 1rem;
    
    .avatar {
      width: 48px;
      height: 48px;
      border-radius: 0.75rem;
      border: 2px solid var(--border);
    }
    
    .user-details {
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
      
      .username {
        font-weight: 600;
        color: var(--text);
        font-size: 1.1rem;
      }
      
      .status {
        color: var(--success);
        font-size: 0.875rem;
      }
    }
  }

  .sign-in-prompt {
    text-align: center;
    padding: 1rem;
    
    .prompt-content {
      max-width: 400px;
      margin: 0 auto;
      
      h3 {
        margin: 0 0 0.5rem 0;
        color: var(--text);
      }
      
      p {
        margin: 0 0 1.5rem 0;
        color: var(--text-muted);
      }
    }
  }

  .game-launcher {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 1rem;
    padding: 2rem;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    
    h2 {
      margin: 0;
      color: var(--text);
      font-size: 1.5rem;
    }
  }

  .installations-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1.5rem;
  }

  .installation-card {
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: 1rem;
    padding: 1.5rem;
    transition: all 0.2s ease;
    
    &:hover {
      transform: translateY(-2px);
      box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
      border-color: var(--primary);
    }
  }

  .installation-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    
    h3 {
      margin: 0;
      color: var(--text);
      font-size: 1.25rem;
    }
    
    .version-badge {
      background: var(--primary);
      color: white;
      padding: 0.25rem 0.75rem;
      border-radius: 1rem;
      font-size: 0.75rem;
      font-weight: 500;
    }
  }

  .installation-details {
    margin-bottom: 1.5rem;
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
    
    .label {
      color: var(--text-muted);
      font-size: 0.875rem;
    }
    
    .value {
      color: var(--text);
      font-size: 0.875rem;
      font-weight: 500;
    }
  }

  .installation-actions {
    .launch-btn {
      width: 100%;
      padding: 0.75rem;
      background: var(--primary);
      color: white;
      border: none;
      border-radius: 0.75rem;
      font-weight: 600;
      font-size: 1rem;
      cursor: pointer;
      transition: all 0.2s ease;
      
      &:hover:not(:disabled) {
        background: var(--primary-hover);
        transform: translateY(-1px);
      }
      
      &:disabled {
        background: var(--surface-variant);
        color: var(--text-muted);
        cursor: not-allowed;
      }
    }
  }

  .no-installations {
    padding: 3rem 1rem;
  }

  .empty-state {
    text-align: center;
    max-width: 400px;
    margin: 0 auto;
    
    .empty-icon {
      font-size: 4rem;
      margin-bottom: 1rem;
    }
    
    h3 {
      margin: 0 0 1rem 0;
      color: var(--text);
    }
    
    p {
      margin: 0 0 2rem 0;
      color: var(--text-muted);
      line-height: 1.5;
    }
  }

  // Button styles
  button {
    border: none;
    border-radius: 0.5rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    
    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }

  .sign-in-btn {
    background: var(--primary);
    color: white;
    padding: 0.75rem 2rem;
    font-size: 1rem;
    
    &:hover {
      background: var(--primary-hover);
      transform: translateY(-1px);
    }
  }

  .sign-out-btn {
    background: var(--surface-variant);
    color: var(--text);
    padding: 0.5rem 1rem;
    
    &:hover {
      background: var(--surface-hover);
    }
  }

  .refresh-btn {
    background: var(--surface-variant);
    color: var(--text);
    padding: 0.5rem 1rem;
    
    &:hover {
      background: var(--surface-hover);
    }
    
    &.primary {
      background: var(--primary);
      color: white;
      padding: 0.75rem 1.5rem;
      
      &:hover {
        background: var(--primary-hover);
      }
    }
  }
</style>
