<!-- @component
AccountSwitcher - Dropdown menu for switching between multiple Microsoft accounts

Displays all available accounts with their status (online/offline/expired).
Allows quick switching between accounts and shows account health indicators.

@example
```svelte
◄AccountSwitcher /►
```
-->
<script lang="ts">
import {
  AuthService,
  currentAccount,
  availableAccounts,
  Icon,
  PlayerHead,
} from "$lib";
import type { LauncherAccount } from "$lib";

let showDropdown = false;

// Show all available accounts, including offline
$: validAccounts = $availableAccounts.filter(
  (acc) => acc?.username !== $currentAccount?.username,
);

// Also log for debugging
$: {
  console.log(
    "🔍 AccountSwitcher - Available accounts:",
    $availableAccounts.length,
    $availableAccounts.map((acc) => ({ ...acc })),
  );
  console.log(
    "🔍 AccountSwitcher - Valid accounts after filtering:",
    validAccounts.length,
    validAccounts,
  );
  if (validAccounts.length > 0) {
    validAccounts.forEach((acc) =>
      console.log(
        "  ✅",
        acc.local_id,
        acc.minecraft_profile?.name || acc.username,
      ),
    );
  }
}

// Determine account status
function getAccountStatus(
  account: LauncherAccount | null,
): "online" | "offline" | "expired" {
  if (!account) return "offline";
  if (!account.access_token) return "offline";
  if (account.access_token_expires_at) {
    const expiryDate = new Date(account.access_token_expires_at);
    if (expiryDate <= new Date()) return "expired";
  }
  return "online";
}

async function switchAccount(account: LauncherAccount) {
  if (account.local_id === $currentAccount?.local_id) return;
  try {
    await AuthService.switchAccount(account.local_id);
    showDropdown = false;
  } catch (error) {
    console.error("Failed to switch account:", error);
  } finally {
    showDropdown = false;
  }
}

$: {
  console.log("🔍 AccountSwitcher - Current account:", $currentAccount);
}
</script>

{#if $currentAccount || $availableAccounts.length > 0}
  <div class="account-switcher">
    <div class="current-account">
      <div
        class="account-avatar-container"
        on:mouseenter={() => (showDropdown = true)}
        on:mouseleave={() => (showDropdown = false)}
        role="button"
        tabindex="0"
      >
        <div
          class="account-avatar minecraft-head"
          title="{$currentAccount?.minecraft_profile?.name ||
            $currentAccount?.username}'s avatar"
        >
          <PlayerHead account={$currentAccount} size={36} />
        </div>
        {#if getAccountStatus($currentAccount) === "online"}
          <div class="status-indicator online" title="Online"></div>
        {:else if getAccountStatus($currentAccount) === "offline"}
          <div class="status-indicator offline" title="Offline"></div>
        {:else}
          <div class="status-indicator expired" title="Token Expired"></div>
        {/if}
      </div>

      <div class="account-info">
        <span class="username"
          >{$currentAccount?.minecraft_profile?.name ||
            $currentAccount?.username ||
            "Unknown User"}</span
        >
        <span class="account-type">
          {#if getAccountStatus($currentAccount) === "offline"}
            Offline Account
          {:else if getAccountStatus($currentAccount) === "expired"}
            Microsoft Account (Token Expired)
          {:else}
            Microsoft Account
          {/if}
        </span>
      </div>

      <div class="dropdown-chevron" class:rotated={showDropdown}>
        <Icon
          name={showDropdown ? "chevron-up" : "chevron-down"}
          forceType="svg"
        />
      </div>

      <div class="dropdown-menu">
        {#each validAccounts as account (account.local_id)}
          <div
            class="account-item"
            class:active={account.local_id === $currentAccount?.local_id}
          >
            <button
              class="account-button"
              on:click={() => switchAccount(account)}
            >
              <div class="account-avatar-container">
                <div
                  class="account-avatar minecraft-head"
                  title="{account.minecraft_profile?.name ||
                    account.username}'s avatar"
                >
                  <PlayerHead {account} size={36} />
                </div>
                {#if getAccountStatus(account) === "online"}
                  <div class="status-indicator online" title="Online"></div>
                {:else if getAccountStatus(account) === "offline"}
                  <div class="status-indicator offline" title="Offline"></div>
                {:else}
                  <div
                    class="status-indicator expired"
                    title="Token Expired"
                  ></div>
                {/if}
              </div>

              <div class="account-info">
                <span class="username"
                  >{account.minecraft_profile?.name ||
                    account.username ||
                    "Unknown User"}</span
                >
                <span class="account-type">
                  {#if getAccountStatus(account) === "offline"}
                    Offline Account
                  {:else if getAccountStatus(account) === "expired"}
                    Microsoft Account (Token Expired)
                  {:else}
                    Microsoft Account
                  {/if}
                </span>
              </div>
            </button>
          </div>
        {/each}
        <div class="account-item add-account-item">
          <button
            class="account-button add-account-btn"
            on:click={() => AuthService.signOut()}
          >
            <div class="account-avatar-container">
              <div class="account-avatar minecraft-head" title="Add Account">
                <span class="avatar-letter">+</span>
              </div>
            </div>
            <div class="account-info">
              <span class="username">Add Account</span>
              <span class="account-type"
                >Sign in with another Microsoft account</span
              >
            </div>
          </button>
        </div>
      </div>
    </div>
  </div>
{:else}
  <div class="no-account-container">
    <button class="sign-in-btn" on:click={() => AuthService.signIn()}>
      <div class="sign-in-avatar">
        <Icon name="user-plus" size="lg" />
      </div>
      <div class="sign-in-info">
        <span class="sign-in-text">Sign in to Microsoft</span>
        <span class="sign-in-help"
          >Access online features and view your Minecraft profile.</span
        >
      </div>
      <Icon name="arrow-right" />
    </button>
  </div>
{/if}

<style lang="scss">
@use "@kablan/clean-ui/scss/variables" as *;

.account-switcher {
  position: relative;
  display: inline-block;
  min-width: 15rem;
  width: 100%;
}

.current-account {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  width: 100%;
  background: var(--container);
  border: 0.0625rem solid var(--dark-600);
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  font-family: inherit;
  color: inherit;

  &:hover:not(:disabled) {
    background: var(--button-hover);
    border-color: var(--primary);
    box-shadow: 0 0.125rem 0.5rem
      color-mix(in srgb, var(--primary), 15%, transparent);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  // show the dropdown-menu when hovered / focused
  &:hover .dropdown-menu {
    display: block;
  }
}

.account-avatar-container {
  position: relative;
  flex-shrink: 0;
}

.account-avatar {
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 50%;
  background: var(--container);
  border: 0.125rem solid var(--dark-600);
  transition: border-color 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 1rem;
  color: var(--text);
  overflow: hidden;

  &.minecraft-head {
    background: linear-gradient(135deg, var(--primary), var(--primary-600));
    color: white;
    border-color: var(--dark-600);
  }

  .avatar-letter {
    user-select: none;
  }
}

.status-indicator {
  position: absolute;
  bottom: -0.125rem;
  right: -0.125rem;
  width: 0.75rem;
  height: 0.75rem;
  border-radius: 50%;
  border: 0.125rem solid var(--container);

  &.online {
    background: var(--green);
  }

  &.offline {
    background: var(--yellow);
  }

  &.expired {
    background: var(--red);
  }
}

.account-info {
  flex: 1;
  text-align: left;
  min-width: 0;
}

.username {
  display: block;
  font-weight: 600;
  color: var(--text);
  font-size: 0.875rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.account-type {
  display: block;
  font-size: 0.75rem;
  color: var(--placeholder);
  margin-top: 0.125rem;
}

.dropdown-chevron {
  flex-shrink: 0;
  fill: var(--placeholder);
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);

  &.rotated {
    transform: rotate(180deg);
  }
}

.dropdown-menu {
  display: none;
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--card);
  border: 0.0625rem solid var(--dark-600);
  border-radius: 0.75rem;
  box-shadow:
    0 0.625rem 1.5625rem rgba(0, 0, 0, 0.3),
    0 0 0 0.0625rem rgba(255, 255, 255, 0.05);
  z-index: 1000;
  overflow: hidden;
  backdrop-filter: blur(1.25rem);
}

.account-item {
  position: relative;
  border-radius: 0.5rem;
  overflow: hidden;

  &.active {
    background: color-mix(in srgb, var(--primary), 10%, transparent);
  }

  &.add-account-item {
    .account-button {
      &:hover:not(:disabled) {
        background: var(--container);
        transform: none;
      }
    }
  }
}

.account-button {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 1rem;
  width: 100%;
  border: none;
  background: transparent;
  cursor: pointer;
  transition: background-color 0.2s ease;
  text-align: left;
  font-family: inherit;
  color: inherit;
  border-radius: 0.5rem;

  &:hover:not(:disabled) {
    background: var(--container);
    transform: none;
  }

  &:disabled {
    opacity: 0.6;
    cursor: default;
  }
}

@media (max-width: 48rem) {
  .account-switcher {
    min-width: 12.5rem;
  }

  .dropdown-menu {
    left: -0.5rem;
    right: -0.5rem;
  }
}
</style>
