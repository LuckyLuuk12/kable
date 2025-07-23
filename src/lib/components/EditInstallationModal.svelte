<script lang="ts">
  import { InstallationManager } from '$lib/managers/InstallationManager';
  import type { KableInstallation } from '$lib/types';
  export let installation: KableInstallation;
  let edited: KableInstallation = { ...installation };
  let javaArgsString: string = edited.java_args.join(' ');
  let dialogRef: HTMLDialogElement;

  function open() {
    dialogRef?.showModal();
  }
  function close() {
    dialogRef?.close();
  }

  function handleInput(e: Event, field: keyof KableInstallation) {
    const target = e.target as HTMLInputElement;
    edited = { ...edited, [field]: target.value };
  }

  function handleJavaArgsInput(e: Event) {
    const target = e.target as HTMLInputElement;
    javaArgsString = target.value;
  }

  async function confirmEdit() {
    edited.java_args = javaArgsString.split(' ').filter(arg => arg.length > 0);
    await InstallationManager.updateInstallation(edited.id, edited);
    close();
  }

  function cancelEdit() {
    close();
  }
</script>

<dialog bind:this={dialogRef} class="edit-installation-modal">
  <h2>Edit Installation</h2>
  <form on:submit|preventDefault={confirmEdit}>
    <label>
      Name:
      <input type="text" bind:value={edited.name} on:input={(e) => handleInput(e, 'name')} />
    </label>
    <label>
      Icon:
      <input type="text" bind:value={edited.icon} on:input={(e) => handleInput(e, 'icon')} />
    </label>
    <label>
      Java Args:
      <input type="text" bind:value={javaArgsString} on:input={handleJavaArgsInput} />
    </label>
    <!-- Add more fields as needed -->
    <div class="actions">
      <button type="submit" class="btn btn-primary">Confirm</button>
      <button type="button" class="btn btn-secondary" on:click={cancelEdit}>Cancel</button>
    </div>
  </form>
</dialog>

<style lang="scss">
@use '@kablan/clean-ui/scss/_variables.scss' as *;

.edit-installation-modal {
  padding: 2rem;
  background: $container;
  border-radius: $border-radius;
  max-width: 28rem;
  margin: 0 auto;
  h2 {
    margin-bottom: 1rem;
    color: $text;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    label {
      color: $text;
      font-size: 1rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
    .actions {
      display: flex;
      gap: 1rem;
      button {
        padding: 0.5rem 1.5rem;
        border-radius: $border-radius;
        border: none;
        font-size: 1rem;
        cursor: pointer;
      }
    }
  }
}
</style>
