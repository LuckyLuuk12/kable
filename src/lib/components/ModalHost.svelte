<script lang="ts">
import { app } from "$lib";
import { type ModalContext, MODAL_CONTEXT } from "$lib/utils/modal";
import { setContext } from "svelte";

function createContext(id: string): ModalContext {
  return {
    resolve(value: unknown) {
      app.resolve(id, value);
    },

    dismiss() {
      app.dismiss(id);
    },
  };
}
</script>

<div
  role="dialog"
  tabindex="-1"
  class="modal-root"
  onkeydown={(e) => {
    if (e.key === "Escape") {
      e.preventDefault();
      app.dismissTop();
    }
  }}>
  {#each app.stack as instance, index (instance.id)}
    {@const context = createContext(instance.id)}
    {@const _ = setContext(MODAL_CONTEXT, context)}

    <div
      class="modal-backdrop"
      style:z-index={5000 + index * 2}
      onclick={() => app.dismiss(instance.id)}
      onkeydown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          app.dismiss(instance.id);
        }
      }}
      role="button"
      tabindex="0">
    </div>

    <div class="modal-container" style:z-index={5001 + index * 2}>
      <instance.component {...instance.props} />
    </div>
  {/each}
</div>

<style>
.modal-root {
  position: fixed;
  inset: 0;
  z-index: 2147483647;
  isolation: isolate;
  pointer-events: none;
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgb(0 0 0 / 50%);
  pointer-events: auto;
}

.modal-container {
  position: fixed;
  inset: 0;

  display: flex;
  align-items: center;
  justify-content: center;

  pointer-events: none;
}

.modal-container > :global(*) {
  pointer-events: auto;
}
</style>
