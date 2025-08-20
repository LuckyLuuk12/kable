<script lang="ts">
  import { onMount } from 'svelte';
  import { Icon, AccountSwitcher, AccountManager } from '$lib';

  // State variables
  let error: string | null = null;
  let userStats = {
    totalPlaytime: 0,
    lastPlayed: null as string | null,
    favoriteDimension: 'Overworld',
    worldsCreated: 0
  };

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
    <!-- Top Row: Account Switcher and Account Management -->
    <div class="top-row">
      <!-- Account Switcher Section -->
      <section class="profile-section">
        <div class="section-header">
          <h2><Icon name="user" forceType="svg" /> Quick Account Switcher</h2>
        </div>
        <AccountSwitcher />
      </section>

      <!-- Account Management Section -->
      <section class="profile-section">
        <div class="section-header">
          <h2><Icon name="user-plus" forceType="svg" /> Account Management</h2>
        </div>
        
        <div class="account-management-container">
          <AccountManager />
        </div>
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
    width: 100%;
    padding: 0 2rem;
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
      background: linear-gradient(135deg, var(--primary), var(--tertiary));
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
    
    p {
      margin: 0;
      color: var(--placeholder);
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
    background: var(--container);
    border: 1px solid var(--dark-600);
    border-radius: var(--border-radius-large);
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
      background: linear-gradient(90deg, transparent, rgba(var(--primary), 0.3), transparent);
    }
    
    &:hover {
      border-color: rgba(var(--primary), 0.3);
      box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
    }

    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1.5rem;
      padding-bottom: 1rem;
      border-bottom: 1px solid rgba(var(--dark-600), 0.5);

      h2 {
        margin: 0;
        font-size: 1.25rem;
        font-weight: 600;
        color: var(--text);
        display: flex;
        align-items: center;
        gap: 0.5rem;
        position: relative;
        word-wrap: break-word;
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
        background: linear-gradient(135deg, rgba(var(--primary), 0.03) 0%, rgba(var(--tertiary), 0.02) 100%);
        border: 1px solid rgba(var(--dark-600), 0.6);
        border-radius: var(--border-radius);
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
          background: linear-gradient(90deg, var(--primary), var(--tertiary));
          transform: translateX(-100%);
          transition: transform 0.3s ease;
        }
        
        &:hover {
          border-color: rgba(var(--primary), 0.3);
          transform: translateY(-2px);
          box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
          
          &::before {
            transform: translateX(0);
          }
        }

        .stat-icon {
          width: 48px;
          height: 48px;
          border-radius: var(--border-radius);
          background: linear-gradient(135deg, rgba(var(--primary), 0.15), rgba(var(--tertiary), 0.1));
          display: flex;
          align-items: center;
          justify-content: center;
          color: var(--primary);
          flex-shrink: 0;
          position: relative;
          
          &::after {
            content: '';
            position: absolute;
            inset: -1px;
            border-radius: var(--border-radius);
            background: linear-gradient(135deg, var(--primary), var(--tertiary));
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
            color: var(--placeholder);
            text-transform: uppercase;
            letter-spacing: 0.5px;
            word-wrap: break-word;
          }

          .stat-value {
            margin: 0;
            font-size: 1.25rem;
            font-weight: 600;
            color: var(--text);
            background: linear-gradient(135deg, var(--primary), var(--tertiary));
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
    background: rgba(var(--red), 0.1);
    border: 1px solid var(--red);
    border-radius: var(--border-radius);
    color: var(--red);
    margin-bottom: 1rem;
  }

  @media (max-width: 768px) {
    .profile-section {
      padding: 1rem;
    }

    .stats-grid {
      grid-template-columns: 1fr !important;
    }
    
    .top-row {
      grid-template-columns: 1fr !important;
    }
  }
</style>
