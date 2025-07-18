<script lang="ts">
  import { onMount } from 'svelte';
  import { AuthManager, currentAccount, availableAccounts, type LauncherAccount } from '$lib';
  import Icon from './Icon.svelte';
  import AuthenticationFlow from './AuthenticationFlow.svelte';

  let isLoading = false;
  let showAddAccount = false;

  // Filter accounts: exclude offline fallbacks (zero UUIDs) but include real offline accounts
  $: validAccounts = $availableAccounts.filter(acc => 
    acc?.local_id &&
    acc.minecraft_profile?.id && acc.minecraft_profile.id !== '00000000-0000-0000-0000-000000000000' &&
    acc.minecraft_profile?.name && acc.minecraft_profile.name.trim() !== ''
  );
  
  // Also log for debugging
  $: {
    console.log('🔍 AccountManager - Available accounts:', $availableAccounts.length);
    console.log('🔍 AccountManager - Valid accounts after filtering:', validAccounts.length);
    if (validAccounts.length > 0) {
      validAccounts.forEach(acc => console.log('  ✅', acc.minecraft_profile?.name || acc.username, 'Local ID:', acc.local_id));
    }
  }
  
  // Check if current account is the offline fallback
  $: isCurrentAccountFallback = $currentAccount && (
    !$currentAccount.minecraft_profile?.id || 
    $currentAccount.minecraft_profile.id === '00000000-0000-0000-0000-000000000000' ||
    !$currentAccount.minecraft_profile?.name ||
    $currentAccount.minecraft_profile.name.trim() === ''
  );
  
  // Determine account status
  function getAccountStatus(account: LauncherAccount): 'online' | 'offline' | 'expired' {
    if (!account.access_token) return 'offline';
    if (account.access_token_expires_at) {
      const expiryDate = new Date(account.access_token_expires_at);
      if (expiryDate <= new Date()) return 'expired';
    }
    return 'online';
  }
  
  // Format token expiry for display
  function formatTokenExpiry(account: LauncherAccount): string {
    if (!account.access_token_expires_at) return 'Never expires';
    const expiryDate = new Date(account.access_token_expires_at);
    const now = new Date();
    const diff = expiryDate.getTime() - now.getTime();
    if (diff <= 0) return 'Expired';
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    if (days > 0) return `Expires in ${days} day${days > 1 ? 's' : ''}`;
    if (hours > 0) return `Expires in ${hours} hour${hours > 1 ? 's' : ''}`;
    return 'Expires soon';
  }

  onMount(async () => {
    // Initialize authentication and load accounts
    await AuthManager.initialize();
    await AuthManager.refreshAvailableAccounts();
  });

  /**
   * Switch to a different account
   */
  async function switchAccount(account: LauncherAccount) {
    if (account.local_id === $currentAccount?.local_id) return;
    isLoading = true;
    try {
      await AuthManager.switchAccount(account.local_id);
    } catch (error) {
      console.error('Failed to switch account:', error);
    } finally {
      isLoading = false;
    }
  }

  /**
   * Remove an account
   */
  async function removeAccount(account: LauncherAccount, event: Event) {
    event.stopPropagation();
    if (!confirm(`Remove "${account.minecraft_profile?.name || account.username}" from your accounts?`)) {
      return;
    }
    isLoading = true;
    try {
      await AuthManager.removeAccount(account.local_id);
    } catch (error) {
      console.error('Failed to remove account:', error);
    } finally {
      isLoading = false;
    }
  }

  /**
   * Refresh current account token
   */
  async function refreshToken() {
    if (!$currentAccount || isCurrentAccountFallback) return;
    
    isLoading = true;
    try {
      await AuthManager.refreshCurrentAccount();
    } catch (error) {
      console.error('Token refresh failed:', error);
    } finally {
      isLoading = false;
    }
  }

  /**
   * Sign out current account
   */
  async function signOut() {
    try {
      await AuthManager.signOut();
    } catch (error) {
      console.error('Sign out failed:', error);
    }
  }

  /**
   * Start sign in flow
   */
  async function startSignIn() {
    isLoading = true;
    try {
      await AuthManager.signIn();
    } catch (error) {
      console.error('Sign in failed:', error);
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="account-manager">
  {#if $currentAccount && !isCurrentAccountFallback}
    <!-- Valid Current Account Display -->
    <div class="current-account-section">
      <div class="section-header">
        <h3>Current Account</h3>
        <div class="account-actions">
          {#if getAccountStatus($currentAccount) === 'expired'}
            <button on:click={refreshToken} class="btn btn-primary btn-sm" disabled={isLoading}>
              <Icon name="refresh" size="sm" />
              {isLoading ? 'Refreshing...' : 'Refresh Token'}
            </button>
          {/if}
          <button on:click={signOut} class="btn btn-secondary btn-sm">
            <Icon name="logout" size="sm" />
            Sign Out
          </button>
        </div>
      </div>

      <div class="current-account-card">
        <div class="account-avatar-container">
        <div class="account-avatar minecraft-head large" title="{$currentAccount.minecraft_profile?.name || $currentAccount.username}'s avatar">
          <span class="avatar-letter">{($currentAccount.minecraft_profile?.name || $currentAccount.username || 'U').charAt(0).toUpperCase()}</span>
          </div>
          
          {#if getAccountStatus($currentAccount) === 'online'}
            <div class="status-indicator online" title="Online"></div>
          {:else if getAccountStatus($currentAccount) === 'offline'}
            <div class="status-indicator offline" title="Offline"></div>
          {:else}
            <div class="status-indicator expired" title="Token Expired"></div>
          {/if}
        </div>

        <div class="account-details">
          <h4>{$currentAccount.minecraft_profile?.name || $currentAccount.username || 'Unknown User'}</h4>
          <p class="account-type">
            {#if getAccountStatus($currentAccount) === 'offline'}
              Offline Account
            {:else if getAccountStatus($currentAccount) === 'expired'}
              Microsoft Account (Token Expired)
            {:else}
              Microsoft Account
            {/if}
          </p>
          <p class="account-id">UUID: {$currentAccount.minecraft_profile?.id}</p>
          
          {#if getAccountStatus($currentAccount) !== 'offline'}
            <p class="token-status" class:expired={getAccountStatus($currentAccount) === 'expired'}>
              {formatTokenExpiry($currentAccount)}
            </p>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- Available Accounts Section -->
  {#if validAccounts.length > 0}
    <div class="available-accounts-section">
      <div class="section-header">
        <h3>Available Accounts</h3>
        <span class="account-count">{validAccounts.length} account{validAccounts.length !== 1 ? 's' : ''}</span>
      </div>

      <div class="accounts-grid">
        {#each validAccounts as account (account.local_id)}
          <div class="account-item" class:active={account.local_id === $currentAccount?.local_id && !isCurrentAccountFallback}>
            <button 
              class="account-button"
              on:click={() => switchAccount(account)}
              disabled={isLoading || (account.local_id === $currentAccount?.local_id && !isCurrentAccountFallback)}
            >
              <div class="account-avatar-container">
            <div class="account-avatar minecraft-head" title="{account.minecraft_profile?.name || account.username}'s avatar">
              <span class="avatar-letter">{(account.minecraft_profile?.name || account.username || 'U').charAt(0).toUpperCase()}</span>
                </div>
                
                {#if getAccountStatus(account) === 'online'}
                  <div class="status-indicator online" title="Online"></div>
                {:else if getAccountStatus(account) === 'offline'}
                  <div class="status-indicator offline" title="Offline"></div>
                {:else}
                  <div class="status-indicator expired" title="Token Expired"></div>
                {/if}
                
                {#if account.local_id === $currentAccount?.local_id && !isCurrentAccountFallback}
                  <div class="current-indicator" title="Current account"></div>
                {/if}
              </div>
              
              <div class="account-info">
                <span class="username">{account.minecraft_profile?.name || account.username || 'Unknown User'}</span>
                <div class="account-meta">
                  <span class="account-id">{account.minecraft_profile?.id.slice(0, 8)}...{account.minecraft_profile?.id.slice(-8)}</span>
                  {#if getAccountStatus(account) === 'offline'}
                    <span class="offline-badge">Offline</span>
                  {:else if getAccountStatus(account) === 'expired'}
                    <span class="expired-badge">Expired</span>
                  {/if}
                </div>
              </div>
              
                {#if account.local_id === $currentAccount?.local_id && !isCurrentAccountFallback}
                <div class="current-badge">
                  <svg width="12" height="12" viewBox="0 0 12 12">
                    <path d="M10 3L4.5 8.5 2 6" stroke="currentColor" stroke-width="2" fill="none"/>
                  </svg>
                  Current
                </div>
              {/if}
            </button>
            
            {#if account.local_id !== $currentAccount?.local_id || isCurrentAccountFallback}
              <button 
                class="remove-btn trash-btn" 
                on:click={(e) => removeAccount(account, e)}
                disabled={isLoading}
                title="Remove account"
                aria-label="Remove {account.minecraft_profile?.name || account.username || 'account'}"
              >
                <Icon name="trash" size="sm" />
              </button>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Add Account Section -->
  <div class="add-account-section">
    {#if showAddAccount}
      <div class="auth-flow-container">
        <AuthenticationFlow 
          on:success={() => showAddAccount = false}
          on:cancel={() => showAddAccount = false}
        />
      </div>
    {:else if validAccounts.length === 0 && (!$currentAccount || isCurrentAccountFallback)}
      <!-- No accounts - show primary sign in -->
      <div class="no-accounts-container">
        <div class="welcome-message">
          <div class="welcome-icon">
            <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
              <circle cx="24" cy="16" r="8" stroke="currentColor" stroke-width="2"/>
              <path d="M8 40c0-8.837 7.163-16 16-16s16 7.163 16 16" stroke="currentColor" stroke-width="2"/>
            </svg>
          </div>
          <h3>Welcome to Kable</h3>
          <p>Sign in with your Microsoft account to get started with Minecraft.</p>
        </div>
        
        <button 
          class="btn btn-primary btn-large sign-in-btn"
          on:click={startSignIn}
          disabled={isLoading}
        >
          <Icon name="microsoft" size="md" />
          {isLoading ? 'Signing in...' : 'Sign in with Microsoft'}
        </button>
      </div>
    {:else}
      <!-- Additional account button -->
      <button 
        class="btn btn-outline add-account-btn"
        on:click={() => showAddAccount = true}
        disabled={isLoading}
      >
        <Icon name="plus" size="sm" />
        Add Microsoft Account
      </button>
    {/if}
  </div>
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;
  
  .account-manager {
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 800px;
    margin: 0 auto;
  }
  
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid $dark-600;
    
    h3 {
      margin: 0;
      font-size: 18px;
      font-weight: 600;
      color: $text;
    }
    
    .account-count {
      font-size: 12px;
      color: $placeholder;
      background: $container;
      padding: 4px 10px;
      border-radius: 6px;
    }
    
    .account-actions {
      display: flex;
      gap: 8px;
    }
  }
  
  .current-account-card {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 16px;
    padding: 24px;
    display: flex;
    align-items: center;
    gap: 20px;
  }
  
  .account-avatar-container {
    position: relative;
    flex-shrink: 0;
  }
  
  .account-avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-weight: 600;
    color: white;
    border: 3px solid $dark-600;
    transition: border-color 0.2s ease;
    
    &.minecraft-head {
      background: linear-gradient(135deg, $primary, $primary-600);
      border-color: $primary;
    }
    
    &.large {
      width: 64px;
      height: 64px;
      font-size: 24px;
    }
    
    width: 40px;
    height: 40px;
    font-size: 16px;
    
    .avatar-letter {
      user-select: none;
    }
  }
  
  .status-indicator {
    position: absolute;
    bottom: -2px;
    right: -2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 3px solid $container;
    
    &.online {
      background: $green;
    }
    
    &.offline {
      background: $yellow;
    }
    
    &.expired {
      background: $red;
    }
  }
  
  .current-indicator {
    position: absolute;
    top: -2px;
    left: -2px;
    width: 12px;
    height: 12px;
    background: $primary;
    border-radius: 50%;
    border: 2px solid $container;
  }
  
  .account-details {
    flex: 1;
    
    h4 {
      margin: 0 0 4px 0;
      font-size: 20px;
      font-weight: 600;
      color: $text;
    }
    
    .account-type {
      margin: 0 0 8px 0;
      font-size: 14px;
      color: $placeholder;
    }
    
    .account-id {
      margin: 0 0 8px 0;
      font-size: 12px;
      color: $placeholder;
      font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    }
    
    .token-status {
      margin: 0;
      font-size: 13px;
      color: $green;
      font-weight: 500;
      
      &.expired {
        color: $red;
      }
    }
  }
  
  .accounts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }
  
  .account-item {
    position: relative;
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 12px;
    overflow: hidden;
    transition: all 0.2s ease;
    
    &:hover {
      border-color: $primary;
      box-shadow: 0 4px 12px rgba($primary, 0.15);
    }
    
    &.active {
      border-color: $primary;
      background: rgba($primary, 0.05);
    }
  }
  
  .account-button {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    width: 100%;
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    color: inherit;
    transition: background-color 0.2s ease;
    
    &:hover:not(:disabled) {
      background: rgba($primary, 0.05);
    }
    
    &:disabled {
      opacity: 0.6;
      cursor: default;
    }
  }
  
  .account-info {
    flex: 1;
    min-width: 0;
    
    .username {
      display: block;
      font-size: 15px;
      font-weight: 600;
      color: $text;
      margin-bottom: 4px;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
  }
  
  .account-meta {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .account-id {
    font-size: 11px;
    color: $placeholder;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    opacity: 0.8;
  }
  
  .offline-badge, .expired-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .offline-badge {
    background: rgba($yellow, 0.15);
    color: $yellow;
  }
  
  .expired-badge {
    background: rgba($red, 0.15);
    color: $red;
  }
  
  .current-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    padding: 4px 8px;
    background: $primary;
    color: white;
    border-radius: 6px;
    font-weight: 500;
    white-space: nowrap;
    
    svg {
      flex-shrink: 0;
    }
  }
  
  .remove-btn.trash-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: $placeholder;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.2s;
    opacity: 0;
    z-index: 2;
    padding: 0;
    box-shadow: none;
    
    svg {
      width: 18px;
      height: 18px;
      color: $red;
      transition: color 0.2s;
    }
  }
  .account-item:hover .remove-btn.trash-btn {
    opacity: 1;
    svg {
      color: darken($red, 18%);
    }
  }
  
  .no-accounts-container {
    text-align: center;
    padding: 48px 24px;
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 16px;
  }
  
  .welcome-message {
    margin-bottom: 32px;
    
    .welcome-icon {
      width: 64px;
      height: 64px;
      margin: 0 auto 16px;
      background: linear-gradient(135deg, $primary, $primary-600);
      border-radius: 16px;
      display: flex;
      align-items: center;
      justify-content: center;
      color: white;
    }
    
    h3 {
      margin: 0 0 8px 0;
      font-size: 24px;
      font-weight: 600;
      color: $text;
    }
    
    p {
      margin: 0;
      font-size: 16px;
      color: $placeholder;
      line-height: 1.5;
    }
  }
  
  .sign-in-btn {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 24px;
    font-size: 16px;
    font-weight: 600;
    border-radius: 12px;
    margin: 0 auto;
  }
  
  .add-account-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 20px;
    font-size: 14px;
    border-radius: 10px;
    margin: 0 auto;
  }
  
  .auth-flow-container {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 16px;
    overflow: hidden;
  }
  
  /* Responsive design */
  @media (max-width: 768px) {
    .accounts-grid {
      grid-template-columns: 1fr;
    }
    
    .current-account-card {
      flex-direction: column;
      text-align: center;
      gap: 16px;
    }
  }
</style>
