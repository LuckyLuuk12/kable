<script lang="ts">
import { AuthenticationFlow, Icon, PlayerHead, app } from "$lib";

let now = $state(Date.now());
let accountSwitcherOpen = $state(false);
let showAuthFlow = $state(false);

const authService = app.authService;

let currentAccount = $derived(authService.activeAccount);
let accounts = $derived(authService.accounts);

let accountStatus = $derived(authService.getAccountStatus(currentAccount));
let accountDisplayName = $derived(authService.getAccountDisplayName(currentAccount));
let accountAvatarTitle = $derived(authService.getAccountAvatarTitle(currentAccount));
let accountUuid = $derived(authService.getAccountUuid(currentAccount));
let tokenExpiryDisplay = $derived(authService.formatTokenExpiry(currentAccount, now));

let isLoading = $derived(authService.loading || authService.authenticating);

$effect(() => {
  const interval = setInterval(() => {
    now = Date.now();
  }, 1000);

  return () => clearInterval(interval);
});

function getAccountStatusLabel(account: typeof currentAccount) {
  const status = authService.getAccountStatus(account);

  switch (status) {
    case "online":
      return "Online";
    case "expired":
      return "Expired";
    default:
      return "Offline";
  }
}

function getAccountStatusClass(account: typeof currentAccount) {
  return authService.getAccountStatus(account);
}

async function selectAccount(account: NonNullable<typeof currentAccount>) {
  if (account.local_id === currentAccount?.local_id) {
    accountSwitcherOpen = false;
    return;
  }

  try {
    await authService.setActive(account);
    accountSwitcherOpen = false;
  } catch (error) {
    console.error("Failed to switch account:", error);
  }
}

function openAddAccount() {
  accountSwitcherOpen = false;
  showAuthFlow = true;
}

function closeAuthFlow() {
  showAuthFlow = false;
}

async function removeCurrentAccount() {
  if (!currentAccount) return;

  if (!confirm(`Remove "${accountDisplayName}" from your accounts?`)) {
    return;
  }

  try {
    await authService.removeActiveAccount();
  } catch (error) {
    console.error("Failed to remove account:", error);
  }
}
</script>

<svelte:options runes={true} />

<div class="account-manager">
  {#if currentAccount}
    <div class="account-section">
      <div class="account-header">
        <div class="account-switcher">
          <button
            class="account-switcher-button"
            class:open={accountSwitcherOpen}
            onclick={() => (accountSwitcherOpen = !accountSwitcherOpen)}
            aria-expanded={accountSwitcherOpen}
            aria-haspopup="listbox"
          >
            <div class="account-switcher-avatar">
              <PlayerHead account={currentAccount} size={40} />
            </div>

            <div class="account-switcher-details">
              <span class="account-switcher-name">{accountDisplayName}</span>
              <span class="account-switcher-status">
                {getAccountStatusLabel(currentAccount)}
              </span>
            </div>

            <Icon name="chevron-down" size="sm" />
          </button>

          {#if accountSwitcherOpen}
            <div class="account-switcher-menu" role="listbox">
              <div class="account-switcher-label">Accounts</div>

              {#each accounts as account (account.local_id)}
                <button
                  class="account-option"
                  class:active={account.local_id === currentAccount.local_id}
                  onclick={() => selectAccount(account)}
                  role="option"
                  aria-selected={account.local_id === currentAccount.local_id}
                >
                  <div class="account-option-avatar">
                    <PlayerHead {account} size={36} />
                    <span
                      class="account-option-status"
                      class:online={getAccountStatusClass(account) === "online"}
                      class:offline={getAccountStatusClass(account) === "offline"}
                      class:expired={getAccountStatusClass(account) === "expired"}
                    ></span>
                  </div>

                  <div class="account-option-details">
                    <span class="account-option-name">
                      {authService.getAccountDisplayName(account)}
                    </span>
                    <span class="account-option-status-text">
                      {getAccountStatusLabel(account)}
                    </span>
                  </div>

                  {#if account.local_id === currentAccount.local_id}
                    <Icon name="check" size="sm" />
                  {/if}
                </button>
              {/each}

              <div class="account-switcher-separator"></div>

              <button class="account-option add-account" onclick={openAddAccount}>
                <div class="account-option-add-icon">
                  <Icon name="plus" size="sm" />
                </div>

                <div class="account-option-details">
                  <span class="account-option-name">Add account</span>
                  <span class="account-option-status-text"> Sign in with Microsoft </span>
                </div>
              </button>
            </div>
          {/if}
        </div>

        <div class="account-actions">
          <button class="btn btn-secondary icon-button" onclick={removeCurrentAccount} disabled={isLoading} title="Remove account" aria-label="Remove account">
            <Icon name="trash" size="sm" />
          </button>
        </div>
      </div>

      <div class="current-account-card">
        <div class="account-avatar-container">
          <div class="account-avatar minecraft-head large" title={accountAvatarTitle}>
            <PlayerHead account={currentAccount} size={64} />
          </div>

          <div
            class="status-indicator"
            class:online={accountStatus === "online"}
            class:offline={accountStatus === "offline"}
            class:expired={accountStatus === "expired"}
            title={getAccountStatusLabel(currentAccount)}
          ></div>
        </div>

        <div class="account-details">
          <div class="account-details-main">
            <h4>{accountDisplayName}</h4>
            <span class="account-uuid">{accountUuid}</span>
          </div>

          <div class="account-details-status">
            {#if accountStatus === "online"}
              <span class="token-status">
                {tokenExpiryDisplay}
              </span>
            {:else if accountStatus === "expired"}
              <span class="token-status expired"> Token expired </span>
            {:else}
              <span class="token-status offline"> Offline </span>
            {/if}
          </div>
        </div>
      </div>

      {#if showAuthFlow}
        <div class="auth-flow-container">
          <div class="auth-flow-header">
            <div>
              <h4>Add account</h4>
              <p>Sign in with another Microsoft account.</p>
            </div>

            <button class="btn btn-secondary icon-button" onclick={closeAuthFlow} aria-label="Close authentication">
              <Icon name="x" size="sm" />
            </button>
          </div>

          <AuthenticationFlow />
        </div>
      {/if}
    </div>
  {:else}
    <div class="no-accounts-container">
      <div class="welcome-message">
        <div class="welcome-icon">
          <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
            <circle cx="24" cy="16" r="8" stroke="currentColor" stroke-width="2" />
            <path d="M8 40c0-8.837 7.163-16 16-16s16 7.163 16 16" stroke="currentColor" stroke-width="2" />
          </svg>
        </div>

        <h3>Welcome to Kable</h3>
        <p>Sign in with your Microsoft account to get started with Minecraft.</p>
      </div>

      <div class="auth-flow-container">
        <AuthenticationFlow />
      </div>
    </div>
  {/if}
</div>

<style lang="scss">
.account-manager {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.account-section {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.account-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.account-switcher {
  position: relative;
  width: min(360px, 100%);
}

.account-switcher-button {
  width: 100%;
  min-height: 58px;
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.55rem 0.75rem;
  background: $color-surface-3;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  color: $color-text;
  cursor: pointer;
  text-align: left;
  transition:
    border-color 0.15s ease,
    background 0.15s ease,
    box-shadow 0.15s ease;

  &:hover,
  &.open {
    border-color: color-mix(in srgb, $color-accent 55%, $color-border);
    background: color-mix(in srgb, $color-surface-3 92%, $color-accent);
  }

  &.open {
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-accent 8%, transparent);
  }

  > :last-child {
    margin-left: auto;
    color: $color-text-muted;
    transition: transform 0.15s ease;
  }

  &.open > :last-child {
    transform: rotate(180deg);
    color: $color-accent;
  }
}

.account-switcher-avatar {
  width: 42px;
  height: 42px;
  flex-shrink: 0;
  overflow: hidden;
  border: 1px solid $color-border;
  border-radius: 50%;
  background: $color-surface-2;
}

.account-switcher-details {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 0.2rem;
}

.account-switcher-name {
  color: $color-text;
  font-size: 0.9rem;
  font-weight: 600;
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.account-switcher-status {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  color: $color-text-muted;
  font-size: 0.7rem;

  &::before {
    content: "";
    width: 6px;
    height: 6px;
    flex-shrink: 0;
    border-radius: 50%;
    background: $color-success;
  }
}

.account-switcher-menu {
  position: absolute;
  top: calc(100% + 0.5rem);
  left: 0;
  right: 0;
  z-index: 100;
  display: flex;
  flex-direction: column;
  padding: 0.45rem;
  background: $color-surface-3;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  box-shadow:
    0 16px 40px rgba(0, 0, 0, 0.28),
    0 4px 12px rgba(0, 0, 0, 0.12);
  backdrop-filter: blur(1rem) saturate(1.2);
}

.account-switcher-label {
  padding: 0.5rem 0.65rem 0.45rem;
  color: $color-text-muted;
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.07em;
}

.account-option {
  width: 100%;
  min-height: 52px;
  display: flex;
  align-items: center;
  gap: 0.7rem;
  padding: 0.5rem 0.6rem;
  background: transparent;
  border: 1px solid transparent;
  border-radius: calc($radius-md - 2px);
  color: $color-text;
  cursor: pointer;
  text-align: left;
  transition:
    background 0.12s ease,
    border-color 0.12s ease;

  &:hover {
    background: color-mix(in srgb, $color-surface-2 70%, transparent);
  }

  &.active {
    background: color-mix(in srgb, $color-accent 8%, transparent);
    border-color: color-mix(in srgb, $color-accent 18%, transparent);
  }

  > :last-child {
    margin-left: auto;
    color: $color-accent;
  }

  &.add-account {
    color: $color-accent;

    &:hover {
      background: color-mix(in srgb, $color-accent 8%, transparent);
    }
  }
}

.account-option-avatar {
  position: relative;
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  border-radius: 50%;
  overflow: visible;
}

.account-option-status {
  position: absolute;
  right: -1px;
  bottom: -1px;
  width: 9px;
  height: 9px;
  border: 2px solid $color-surface-3;
  border-radius: 50%;

  &.online {
    background: $color-success;
  }

  &.offline {
    background: $color-warning;
  }

  &.expired {
    background: $color-error;
  }
}

.account-option-details {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 0.15rem;
}

.account-option-name {
  color: $color-text;
  font-size: 0.85rem;
  font-weight: 500;
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.account-option-status-text {
  color: $color-text-muted;
  font-size: 0.68rem;
}

.account-switcher-separator {
  height: 1px;
  margin: 0.4rem 0;
  background: $color-border;
}

.account-option-add-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: $color-accent;
  border: 1px dashed color-mix(in srgb, $color-accent 40%, $color-border);
  border-radius: 50%;
  background: color-mix(in srgb, $color-accent 5%, transparent);
}

.account-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.icon-button {
  width: 2.35rem;
  height: 2.35rem;
  min-width: 2.35rem;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Current account */

.current-account-card {
  position: relative;
  min-height: 190px;
  display: flex;
  align-items: center;
  gap: 2rem;
  padding: 2rem 2.25rem;
  overflow: hidden;
  background: radial-gradient(circle at 0% 100%, color-mix(in srgb, $color-accent 9%, transparent), transparent 45%), $color-surface-3;
  border: 1px solid $color-border;
  border-radius: $radius-md;

  &::after {
    content: "";
    position: absolute;
    top: 0;
    right: 0;
    width: 35%;
    height: 100%;
    pointer-events: none;
    background: linear-gradient(90deg, transparent, color-mix(in srgb, $color-accent 3%, transparent));
  }
}

.account-avatar-container {
  position: relative;
  flex-shrink: 0;
  z-index: 1;
}

.account-avatar {
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border: 3px solid $color-border;
  border-radius: 50%;
  box-shadow:
    0 8px 24px rgba(0, 0, 0, 0.2),
    0 0 0 6px color-mix(in srgb, $color-accent 5%, transparent);

  &.minecraft-head {
    background: linear-gradient(135deg, $color-accent, $color-accent-secondary);
  }

  &.large {
    width: 96px;
    height: 96px;
  }
}

.status-indicator {
  position: absolute;
  right: 1px;
  bottom: 1px;
  width: 19px;
  height: 19px;
  border: 4px solid $color-surface-3;
  border-radius: 50%;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.25);

  &.online {
    background: $color-success;
  }

  &.offline {
    background: $color-warning;
  }

  &.expired {
    background: $color-error;
  }
}

.account-details {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 2rem;
  min-width: 0;
  flex: 1;
}

.account-details-main {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 0.5rem;

  h4 {
    margin: 0;
    color: $color-text;
    font-size: 1.45rem;
    font-weight: 650;
    line-height: 1.2;
  }
}

.account-uuid {
  width: fit-content;
  max-width: 100%;
  padding: 0.3rem 0.5rem;
  color: $color-text-muted;
  background: $color-surface-2;
  border: 1px solid $color-border;
  border-radius: calc($radius-md - 3px);
  font-family: "Monaco", "Menlo", "Ubuntu Mono", monospace;
  font-size: 0.68rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.account-details-status {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.token-status {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  padding: 0.45rem 0.7rem;
  color: $color-success;
  background: color-mix(in srgb, $color-success 7%, transparent);
  border: 1px solid color-mix(in srgb, $color-success 18%, transparent);
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 500;
  white-space: nowrap;

  &::before {
    content: "";
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }

  &.expired {
    color: $color-error;
    background: color-mix(in srgb, $color-error 7%, transparent);
    border-color: color-mix(in srgb, $color-error 18%, transparent);
  }

  &.offline {
    color: $color-warning;
    background: color-mix(in srgb, $color-warning 7%, transparent);
    border-color: color-mix(in srgb, $color-warning 18%, transparent);
  }
}

/* Authentication */

.auth-flow-container {
  overflow: hidden;
  background: $color-surface-3;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
}

.auth-flow-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1.1rem 1.35rem;
  background: color-mix(in srgb, $color-surface-2 45%, transparent);
  border-bottom: 1px solid $color-border;

  h4 {
    margin: 0 0 0.2rem;
    color: $color-text;
    font-size: 0.95rem;
    font-weight: 600;
  }

  p {
    margin: 0;
    color: $color-text-muted;
    font-size: 0.75rem;
  }
}

/* Empty state */

.no-accounts-container {
  width: 100%;
  min-height: 420px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem;
  text-align: center;
  background: radial-gradient(circle at 50% 0%, color-mix(in srgb, $color-accent 8%, transparent), transparent 45%), $color-surface-3;
  border: 1px solid $color-border;
  border-radius: $radius-md;
}

.welcome-message {
  margin-bottom: 2rem;

  .welcome-icon {
    width: 72px;
    height: 72px;
    margin: 0 auto 1.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    background: linear-gradient(135deg, $color-accent, $color-accent-secondary);
    border-radius: 18px;
    box-shadow: 0 10px 30px color-mix(in srgb, $color-accent 18%, transparent);
  }

  h3 {
    margin: 0 0 0.5rem;
    color: $color-text;
    font-size: 1.4rem;
    font-weight: 650;
  }

  p {
    max-width: 480px;
    margin: 0 auto;
    color: $color-text-muted;
    font-size: 0.85rem;
    line-height: 1.5;
  }
}

.no-accounts-container > .auth-flow-container {
  width: min(520px, 100%);
  text-align: left;
  box-shadow: none;
}

/* Responsive */

@media (max-width: 700px) {
  .account-header {
    align-items: stretch;
  }

  .account-switcher {
    min-width: 0;
    flex: 1;
  }

  .current-account-card {
    min-height: 0;
    align-items: flex-start;
    gap: 1.25rem;
    padding: 1.5rem;
  }

  .account-avatar.large {
    width: 72px;
    height: 72px;
  }

  .account-details {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.8rem;
  }

  .account-details-status {
    align-self: flex-start;
  }
}

@media (max-width: 500px) {
  .account-header {
    flex-direction: column;
  }

  .account-switcher {
    width: 100%;
  }

  .account-actions {
    justify-content: flex-end;
  }

  .current-account-card {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .account-details {
    align-items: center;
    width: 100%;
  }

  .account-details-main {
    align-items: center;
  }

  .account-uuid {
    max-width: 100%;
  }

  .no-accounts-container {
    min-height: 0;
    padding: 2rem 1rem;
  }
}
</style>
