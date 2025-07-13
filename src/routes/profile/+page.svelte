<script lang="ts">
  import { AuthManager, GameManager } from '$lib';
  import { currentAccount } from '$lib/auth';
  import { installations } from '$lib/game';
  import { onMount } from 'svelte';

  let isCreatingProfile = false;
  let newProfileName = '';
  let newProfileVersion = '1.21.1';
  let newProfileModLoader = 'vanilla';

  onMount(async () => {
    await Promise.all([
      AuthManager.initialize(),
      GameManager.initialize()
    ]);
  });

  async function createNewProfile() {
    if (!newProfileName.trim()) return;
    
    try {
      isCreatingProfile = true;
      // This would need to be implemented in GameManager
      console.log('Creating profile:', {
        name: newProfileName,
        version: newProfileVersion,
        modLoader: newProfileModLoader
      });
      
      // Reset form
      newProfileName = '';
      newProfileVersion = '1.21.1';
      newProfileModLoader = 'vanilla';
    } catch (error) {
      console.error('Failed to create profile:', error);
    } finally {
      isCreatingProfile = false;
    }
  }

  async function deleteProfile(profileId: string) {
    if (!confirm('Are you sure you want to delete this profile? This action cannot be undone.')) {
      return;
    }
    
    try {
      // This would need to be implemented in GameManager
      console.log('Deleting profile:', profileId);
    } catch (error) {
      console.error('Failed to delete profile:', error);
    }
  }

  async function editProfile(profileId: string) {
    // This would open an edit modal or navigate to an edit page
    console.log('Editing profile:', profileId);
  }

  async function duplicateProfile(profileId: string) {
    try {
      // This would need to be implemented in GameManager
      console.log('Duplicating profile:', profileId);
    } catch (error) {
      console.error('Failed to duplicate profile:', error);
    }
  }

  // Available Minecraft versions (this would come from an API)
  const availableVersions = [
    '1.21.1', '1.21', '1.20.6', '1.20.4', '1.20.1', '1.19.4', '1.19.2', '1.18.2', '1.17.1', '1.16.5'
  ];

  const modLoaders = [
    { id: 'vanilla', name: 'Vanilla' },
    { id: 'fabric', name: 'Fabric' },
    { id: 'forge', name: 'Forge' },
    { id: 'quilt', name: 'Quilt' },
    { id: 'neoforge', name: 'NeoForge' }
  ];
</script>

<div class="profile-page">
  <div class="page-header">
    <h1>Profiles</h1>
    <p>Manage your Minecraft game profiles</p>
  </div>

  <!-- Account Status -->
  <section class="account-status">
    {#if $currentAccount}
      <div class="account-card">
        <img src={$currentAccount?.avatar_url || '/default-avatar.png'} alt="Avatar" class="account-avatar" />
        <div class="account-info">
          <h3>{$currentAccount.username}</h3>
          <p>UUID: <code>{$currentAccount.uuid}</code></p>
          <span class="account-status-badge">✅ Authenticated</span>
        </div>
      </div>
    {:else}
      <div class="no-account">
        <div class="warning-icon">⚠️</div>
        <div class="warning-content">
          <h3>No account connected</h3>
          <p>Sign in with Microsoft to create and manage profiles</p>
          <button on:click={() => AuthManager.signIn()} class="sign-in-btn">
            Sign in with Microsoft
          </button>
        </div>
      </div>
    {/if}
  </section>

  <!-- Create New Profile -->
  <section class="create-profile">
    <h2>Create New Profile</h2>
    
    <div class="profile-form">
      <div class="form-row">
        <div class="form-group">
          <label for="profile-name">Profile Name</label>
          <input 
            id="profile-name"
            type="text" 
            bind:value={newProfileName}
            placeholder="My Awesome Profile"
            class="profile-input"
          />
        </div>
        
        <div class="form-group">
          <label for="profile-version">Minecraft Version</label>
          <select 
            id="profile-version"
            bind:value={newProfileVersion}
            class="profile-select"
          >
            {#each availableVersions as version}
              <option value={version}>{version}</option>
            {/each}
          </select>
        </div>
        
        <div class="form-group">
          <label for="profile-modloader">Mod Loader</label>
          <select 
            id="profile-modloader"
            bind:value={newProfileModLoader}
            class="profile-select"
          >
            {#each modLoaders as loader}
              <option value={loader.id}>{loader.name}</option>
            {/each}
          </select>
        </div>
      </div>
      
      <button 
        on:click={createNewProfile}
        disabled={!newProfileName.trim() || isCreatingProfile || !$currentAccount}
        class="create-btn"
      >
        {#if isCreatingProfile}
          Creating...
        {:else}
          Create Profile
        {/if}
      </button>
    </div>
  </section>

  <!-- Existing Profiles -->
  <section class="existing-profiles">
    <h2>Your Profiles</h2>
    
    {#if $installations && $installations.length > 0}
      <div class="profiles-grid">
        {#each $installations as installation}
          <div class="profile-card">
            <div class="profile-header">
              <div class="profile-icon">
                {#if installation.mod_loader && installation.mod_loader !== 'vanilla'}
                  🧩
                {:else}
                  📦
                {/if}
              </div>
              <div class="profile-title">
                <h3>{installation.name}</h3>
                <span class="profile-version">{installation.version}</span>
              </div>
            </div>
            
            <div class="profile-details">
              <div class="detail-item">
                <span class="label">Type:</span>
                <span class="value">{installation.installation_type}</span>
              </div>
              
              {#if installation.mod_loader}
                <div class="detail-item">
                  <span class="label">Mod Loader:</span>
                  <span class="value">{installation.mod_loader}</span>
                </div>
              {/if}
              
              <div class="detail-item">
                <span class="label">Last Played:</span>
                <span class="value">Never</span>
              </div>
            </div>
            
            <div class="profile-actions">
              <button 
                on:click={() => editProfile(installation.id)}
                class="action-btn edit-btn"
                title="Edit Profile"
              >
                ✏️
              </button>
              
              <button 
                on:click={() => duplicateProfile(installation.id)}
                class="action-btn duplicate-btn"
                title="Duplicate Profile"
              >
                📋
              </button>
              
              <button 
                on:click={() => deleteProfile(installation.id)}
                class="action-btn delete-btn"
                title="Delete Profile"
              >
                🗑️
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="no-profiles">
        <div class="empty-state">
          <div class="empty-icon">📋</div>
          <h3>No profiles found</h3>
          <p>Create your first profile to get started, or refresh to scan for existing installations.</p>
          <button on:click={() => GameManager.refreshInstallations()} class="refresh-btn">
            🔄 Scan for profiles
          </button>
        </div>
      </div>
    {/if}
  </section>
</div>

<style lang="scss">
  .profile-page {
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

  .account-status {
    margin-bottom: 2rem;
  }

  .account-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1.5rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 1rem;
    
    .account-avatar {
      width: 64px;
      height: 64px;
      border-radius: 1rem;
      border: 2px solid var(--border);
    }
    
    .account-info {
      flex: 1;
      
      h3 {
        margin: 0 0 0.5rem 0;
        color: var(--text);
        font-size: 1.25rem;
      }
      
      p {
        margin: 0 0 0.5rem 0;
        color: var(--text-muted);
        font-size: 0.875rem;
        
        code {
          background: var(--surface-variant);
          padding: 0.25rem 0.5rem;
          border-radius: 0.25rem;
          font-family: 'Fira Code', monospace;
        }
      }
      
      .account-status-badge {
        display: inline-block;
        background: var(--success-light);
        color: var(--success);
        padding: 0.25rem 0.75rem;
        border-radius: 1rem;
        font-size: 0.75rem;
        font-weight: 500;
      }
    }
  }

  .no-account {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1.5rem;
    background: var(--warning-light);
    border: 1px solid var(--warning);
    border-radius: 1rem;
    
    .warning-icon {
      font-size: 2rem;
    }
    
    .warning-content {
      flex: 1;
      
      h3 {
        margin: 0 0 0.5rem 0;
        color: var(--warning);
      }
      
      p {
        margin: 0 0 1rem 0;
        color: var(--text-muted);
      }
    }
  }

  .create-profile {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 1rem;
    padding: 2rem;
    margin-bottom: 2rem;
    
    h2 {
      margin: 0 0 1.5rem 0;
      color: var(--text);
      font-size: 1.5rem;
    }
  }

  .profile-form {
    .form-row {
      display: grid;
      grid-template-columns: 2fr 1fr 1fr;
      gap: 1rem;
      margin-bottom: 1.5rem;
      
      @media (max-width: 768px) {
        grid-template-columns: 1fr;
      }
    }
    
    .form-group {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      
      label {
        font-weight: 500;
        color: var(--text);
        font-size: 0.875rem;
      }
    }
    
    .profile-input, .profile-select {
      padding: 0.75rem;
      border: 1px solid var(--border);
      border-radius: 0.5rem;
      background: var(--background);
      color: var(--text);
      font-size: 0.875rem;
      
      &:focus {
        outline: none;
        border-color: var(--primary);
      }
    }
    
    .create-btn {
      padding: 0.75rem 2rem;
      background: var(--primary);
      color: white;
      border: none;
      border-radius: 0.5rem;
      font-weight: 600;
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

  .existing-profiles {
    h2 {
      margin: 0 0 1.5rem 0;
      color: var(--text);
      font-size: 1.5rem;
    }
  }

  .profiles-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1.5rem;
  }

  .profile-card {
    background: var(--surface);
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

  .profile-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
    
    .profile-icon {
      font-size: 2rem;
      width: 48px;
      height: 48px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: var(--background);
      border-radius: 0.75rem;
      border: 1px solid var(--border);
    }
    
    .profile-title {
      flex: 1;
      
      h3 {
        margin: 0 0 0.25rem 0;
        color: var(--text);
        font-size: 1.125rem;
      }
      
      .profile-version {
        background: var(--primary);
        color: white;
        padding: 0.25rem 0.75rem;
        border-radius: 1rem;
        font-size: 0.75rem;
        font-weight: 500;
      }
    }
  }

  .profile-details {
    margin-bottom: 1.5rem;
  }

  .detail-item {
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

  .profile-actions {
    display: flex;
    gap: 0.5rem;
    
    .action-btn {
      flex: 1;
      padding: 0.5rem;
      border: none;
      border-radius: 0.5rem;
      font-size: 1rem;
      cursor: pointer;
      transition: all 0.2s ease;
      
      &.edit-btn {
        background: var(--primary);
        color: white;
        
        &:hover {
          background: var(--primary-hover);
        }
      }
      
      &.duplicate-btn {
        background: var(--surface-variant);
        color: var(--text);
        
        &:hover {
          background: var(--surface-hover);
        }
      }
      
      &.delete-btn {
        background: var(--error);
        color: white;
        
        &:hover {
          background: var(--error-hover);
        }
      }
    }
  }

  .no-profiles {
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

  .sign-in-btn, .refresh-btn {
    padding: 0.75rem 1.5rem;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: 0.5rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    
    &:hover {
      background: var(--primary-hover);
      transform: translateY(-1px);
    }
  }
</style>
