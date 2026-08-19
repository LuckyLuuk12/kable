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
    <div class="account-content">
      <div class="account-main">
        <div class="account-avatar">
          <PlayerHead account={currentAccount} size={72} />

          <span
            class="status-dot"
            class:online={accountStatus === "online"}
            class:offline={accountStatus === "offline"}
            class:expired={accountStatus === "expired"}
            title={getAccountStatusLabel(currentAccount)}></span>
        </div>

        <div class="account-identity">
          <div class="account-name-row">
            <h3>{accountDisplayName}</h3>

            <span class="status-badge" class:online={accountStatus === "online"} class:offline={accountStatus === "offline"} class:expired={accountStatus === "expired"}>
              {getAccountStatusLabel(currentAccount)}
            </span>
          </div>

          <p class="account-description">
            {#if accountStatus === "online"}
              {tokenExpiryDisplay}
            {:else if accountStatus === "expired"}
              Your authentication token has expired.
            {:else}
              This account is currently offline.
            {/if}
          </p>

          <span class="account-uuid" title={accountUuid}>
            {accountUuid}
          </span>
        </div>

        <div class="account-actions">
          <div class="account-switcher">
            <button
              class="switcher-button"
              class:open={accountSwitcherOpen}
              onclick={() => (accountSwitcherOpen = !accountSwitcherOpen)}
              aria-expanded={accountSwitcherOpen}
              aria-haspopup="listbox">
              <Icon name="user" size="sm" forceType="svg" />
              <span>Switch account</span>
              <Icon name="chevron-down" size="sm" forceType="svg" />
            </button>

            {#if accountSwitcherOpen}
              <div class="switcher-menu" role="listbox">
                <div class="switcher-heading">Your accounts</div>

                {#each accounts as account (account.local_id)}
                  <button
                    class="account-option"
                    class:active={account.local_id === currentAccount.local_id}
                    onclick={() => selectAccount(account)}
                    role="option"
                    aria-selected={account.local_id === currentAccount.local_id}>
                    <div class="option-avatar">
                      <PlayerHead {account} size={36} />

                      <span
                        class="option-status"
                        class:online={getAccountStatusClass(account) === "online"}
                        class:offline={getAccountStatusClass(account) === "offline"}
                        class:expired={getAccountStatusClass(account) === "expired"}></span>
                    </div>

                    <div class="option-details">
                      <span class="option-name">
                        {authService.getAccountDisplayName(account)}
                      </span>

                      <span class="option-status-text">
                        {getAccountStatusLabel(account)}
                      </span>
                    </div>

                    {#if account.local_id === currentAccount.local_id}
                      <Icon name="check" size="sm" />
                    {/if}
                  </button>
                {/each}

                <div class="menu-divider"></div>

                <button class="add-account-button" onclick={openAddAccount}>
                  <span class="add-icon">
                    <Icon name="plus" size="sm" />
                  </span>

                  <span>
                    <strong>Add account</strong>
                    <small>Sign in with Microsoft</small>
                  </span>
                </button>
              </div>
            {/if}
          </div>

          <button class="remove-button" onclick={removeCurrentAccount} disabled={isLoading} title="Remove account" aria-label="Remove account">
            <Icon name="trash" size="sm" />
          </button>
        </div>
      </div>

      <div class="account-meta">
        <div class="meta-item">
          <span class="meta-label">Account</span>
          <span class="meta-value">{accountDisplayName}</span>
        </div>

        <div class="meta-item">
          <span class="meta-label">Authentication</span>

          <span class="meta-value" class:success={accountStatus === "online"} class:warning={accountStatus === "offline"} class:error={accountStatus === "expired"}>
            {#if accountStatus === "online"}
              {tokenExpiryDisplay}
            {:else if accountStatus === "expired"}
              Token expired
            {:else}
              Offline
            {/if}
          </span>
        </div>
      </div>

      {#if showAuthFlow}
        <div class="auth-section">
          <div class="auth-header">
            <div>
              <h4>Add Microsoft account</h4>
              <p>Sign in with another Microsoft account to add it to Kable.</p>
            </div>

            <button class="close-button" onclick={closeAuthFlow} aria-label="Close authentication">
              <Icon name="x" size="sm" />
            </button>
          </div>

          <AuthenticationFlow />
        </div>
      {/if}
    </div>
  {:else}
    <div class="empty-state">
      <div class="empty-icon">
        <Icon name="user-plus" size="lg" />
      </div>

      <div class="empty-content">
        <h3>No Microsoft account connected</h3>
        <p>Sign in with your Microsoft account to launch Minecraft and manage your profiles.</p>
      </div>

      <div class="empty-auth">
        <AuthenticationFlow />
      </div>
    </div>
  {/if}
</div>

<style lang="scss">
.account-manager {
  width: 100%;
}

.account-content {
  display: flex;
  flex-direction: column;
}

.account-main {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 1.25rem;
}

.account-avatar {
  position: relative;

  display: flex;
  align-items: center;
  justify-content: center;

  width: 76px;
  height: 76px;

  flex-shrink: 0;

  overflow: hidden;

  background: $color-surface-2;
  border: 1px solid $color-border;
  border-radius: $radius-md;
}

.status-dot {
  position: absolute;
  right: 3px;
  bottom: 3px;

  width: 13px;
  height: 13px;

  border: 3px solid $color-surface-2;
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

.account-identity {
  min-width: 0;
}

.account-name-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.6rem;

  h3 {
    min-width: 0;

    margin: 0;

    color: $color-text;
    font-size: 1.25rem;
    font-weight: 650;
    line-height: 1.25;

    overflow: hidden;
    text-overflow: ellipsis;
  }
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;

  padding: 0.2rem 0.5rem;

  border: 1px solid;
  border-radius: 999px;

  font-size: 0.65rem;
  font-weight: 600;
  line-height: 1.2;

  &::before {
    content: "";

    width: 5px;
    height: 5px;

    border-radius: 50%;
    background: currentColor;
  }

  &.online {
    color: $color-success;
    background: color-mix(in srgb, $color-success 8%, transparent);
    border-color: color-mix(in srgb, $color-success 20%, transparent);
  }

  &.offline {
    color: $color-warning;
    background: color-mix(in srgb, $color-warning 8%, transparent);
    border-color: color-mix(in srgb, $color-warning 20%, transparent);
  }

  &.expired {
    color: $color-error;
    background: color-mix(in srgb, $color-error 8%, transparent);
    border-color: color-mix(in srgb, $color-error 20%, transparent);
  }
}

.account-description {
  margin: 0.35rem 0 0;

  color: $color-text-muted;
  font-size: 0.8rem;
  line-height: 1.4;
}

.account-uuid {
  display: block;

  max-width: min(100%, 360px);

  margin-top: 0.5rem;

  color: $color-text-muted;

  font-family: "Monaco", "Menlo", "Ubuntu Mono", monospace;
  font-size: 0.65rem;

  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.7;
}

.account-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.account-switcher {
  position: relative;
}

.switcher-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;

  height: 2.35rem;
  padding: 0 0.75rem;

  color: $color-text-muted;
  background: $color-surface-2;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  font-size: 0.75rem;
  font-weight: 500;

  cursor: pointer;

  transition:
    color 0.15s ease,
    border-color 0.15s ease,
    background 0.15s ease;

  &:hover,
  &.open {
    color: $color-text;
    border-color: color-mix(in srgb, $color-accent 35%, $color-border);
    background: color-mix(in srgb, $color-accent 5%, $color-surface-2);
  }

  > :last-child {
    transition: transform 0.15s ease;
  }

  &.open > :last-child {
    transform: rotate(180deg);
    color: $color-accent;
  }
}

.remove-button,
.close-button {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 2.35rem;
  height: 2.35rem;
  padding: 0;

  color: $color-text-muted;
  background: transparent;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  cursor: pointer;

  transition:
    color 0.15s ease,
    border-color 0.15s ease,
    background 0.15s ease;

  &:hover:not(:disabled) {
    color: $color-error;
    border-color: color-mix(in srgb, $color-error 35%, $color-border);
    background: color-mix(in srgb, $color-error 6%, transparent);
  }

  &:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }
}

.switcher-menu {
  position: absolute;

  top: calc(100% + 0.5rem);
  right: 0;

  z-index: 20;

  width: 280px;

  padding: 0.4rem;

  background: $color-surface-3;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  box-shadow:
    0 12px 30px rgba(0, 0, 0, 0.22),
    0 2px 8px rgba(0, 0, 0, 0.12);
}

.switcher-heading {
  padding: 0.5rem 0.6rem;

  color: $color-text-muted;

  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.account-option {
  display: flex;
  align-items: center;
  gap: 0.65rem;

  width: 100%;
  min-height: 52px;

  padding: 0.45rem 0.55rem;

  color: $color-text;
  background: transparent;

  border: 1px solid transparent;
  border-radius: calc($radius-md - 2px);

  cursor: pointer;
  text-align: left;

  transition:
    background 0.12s ease,
    border-color 0.12s ease;

  &:hover {
    background: $color-surface-2;
  }

  &.active {
    background: color-mix(in srgb, $color-accent 7%, transparent);
    border-color: color-mix(in srgb, $color-accent 15%, transparent);

    > :last-child {
      color: $color-accent;
    }
  }
}

.option-avatar {
  position: relative;

  width: 36px;
  height: 36px;

  flex-shrink: 0;

  border-radius: 50%;
}

.option-status {
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

.option-details {
  min-width: 0;
  flex: 1;

  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}

.option-name {
  color: $color-text;

  font-size: 0.8rem;
  font-weight: 500;

  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.option-status-text {
  color: $color-text-muted;
  font-size: 0.65rem;
}

.menu-divider {
  height: 1px;
  margin: 0.35rem 0;

  background: $color-border;
}

.add-account-button {
  display: flex;
  align-items: center;
  gap: 0.65rem;

  width: 100%;

  padding: 0.55rem;

  color: $color-accent;
  background: transparent;

  border: 0;
  border-radius: calc($radius-md - 2px);

  cursor: pointer;
  text-align: left;

  &:hover {
    background: color-mix(in srgb, $color-accent 7%, transparent);
  }

  strong,
  small {
    display: block;
  }

  strong {
    font-size: 0.8rem;
    font-weight: 600;
  }

  small {
    margin-top: 0.1rem;

    color: $color-text-muted;
    font-size: 0.65rem;
  }
}

.add-icon {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 36px;
  height: 36px;

  flex-shrink: 0;

  border: 1px dashed color-mix(in srgb, $color-accent 40%, $color-border);
  border-radius: 50%;
}

/* Secondary information */

.account-meta {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));

  margin-top: 1.25rem;
  padding-top: 1rem;

  border-top: 1px solid $color-border;
}

.meta-item {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;

  min-width: 0;

  &:not(:first-child) {
    padding-left: 1.25rem;
    border-left: 1px solid $color-border;
  }
}

.meta-label {
  color: $color-text-muted;

  font-size: 0.65rem;
  font-weight: 600;

  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.meta-value {
  min-width: 0;

  color: $color-text;

  font-size: 0.8rem;
  font-weight: 500;

  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;

  &.success {
    color: $color-success;
  }

  &.warning {
    color: $color-warning;
  }

  &.error {
    color: $color-error;
  }
}

/* Authentication */

.auth-section {
  margin-top: 1.25rem;

  overflow: hidden;

  background: $color-surface-2;

  border: 1px solid $color-border;
  border-radius: $radius-md;
}

.auth-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;

  padding: 1rem 1.15rem;

  border-bottom: 1px solid $color-border;

  h4 {
    margin: 0 0 0.2rem;

    color: $color-text;
    font-size: 0.85rem;
    font-weight: 600;
  }

  p {
    margin: 0;

    color: $color-text-muted;
    font-size: 0.7rem;
  }
}

/* Empty state */

.empty-state {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 1.25rem;

  padding: 1.5rem;

  background: $color-surface-2;

  border: 1px solid $color-border;
  border-radius: $radius-md;
}

.empty-icon {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 52px;
  height: 52px;

  color: $color-accent;

  background: color-mix(in srgb, $color-accent 8%, transparent);

  border: 1px solid color-mix(in srgb, $color-accent 18%, transparent);
  border-radius: $radius-md;
}

.empty-content {
  min-width: 0;

  h3 {
    margin: 0 0 0.3rem;

    color: $color-text;
    font-size: 0.95rem;
    font-weight: 600;
  }

  p {
    max-width: 500px;

    margin: 0;

    color: $color-text-muted;

    font-size: 0.75rem;
    line-height: 1.5;
  }
}

.empty-auth {
  grid-column: 1 / -1;

  padding-top: 1rem;

  border-top: 1px solid $color-border;
}

/* Responsive */

@media (max-width: 800px) {
  .account-main {
    grid-template-columns: auto minmax(0, 1fr);
  }

  .account-actions {
    grid-column: 2;
  }
}

@media (max-width: 560px) {
  .account-main {
    grid-template-columns: auto minmax(0, 1fr);
    gap: 1rem;
  }

  .account-avatar {
    width: 60px;
    height: 60px;
  }

  .account-name-row {
    h3 {
      font-size: 1.05rem;
    }
  }

  .account-actions {
    grid-column: 1 / -1;

    width: 100%;
  }

  .account-switcher {
    flex: 1;
  }

  .switcher-button {
    width: 100%;
    justify-content: center;
  }

  .remove-button {
    flex-shrink: 0;
  }

  .account-meta {
    grid-template-columns: 1fr;
    gap: 0.75rem;
  }

  .meta-item:not(:first-child) {
    padding-left: 0;
    padding-top: 0.75rem;

    border-top: 1px solid $color-border;
    border-left: 0;
  }

  .empty-state {
    grid-template-columns: 1fr;
    text-align: center;
  }

  .empty-icon {
    margin: 0 auto;
  }

  .empty-auth {
    text-align: left;
  }
}
</style>
