<script lang="ts">
import { app, Image } from "$lib";

let showAds = $derived(app.customizationService.settings?.content?.allow_ads ?? true);

async function openUrl(url: string) {
  await app.openUrl(url);
}
</script>

{#if showAds}
  <div class="advertisement-banner">
    <div class="banner-background">
      <Image key="advertisement-banner" alt="Banner" className="banner-image" width="100%" height="100%" />
    </div>

    <div class="banner-overlay"></div>

    <div class="banner-content">
      <div class="banner-actions">
        <button class="banner-button primary" onclick={() => openUrl("https://kablan.nl")}>
          <Image key="kablan-logo" alt="Kablan" className="button-image" width="auto" height="2.5rem" />
          <span>Kablan.nl</span>
        </button>

        <button class="banner-button secondary" onclick={() => openUrl("https://modrinth.com/mod/luckybindings")}>
          <Image key="luckybindings-logo" alt="LuckyBindings" className="button-image" width="auto" height="2.5rem" />
          <span>LuckyBindings Mod</span>
        </button>

        <button class="banner-button kofi" onclick={() => openUrl("https://ko-fi.com/luckyluuk")}>
          <Image key="kofi-logo" alt="Ko-fi" className="button-image" width="auto" height="2.5rem" />
          <span>Support me on Ko-fi</span>
        </button>
      </div>

      <div class="artist-recruitment">
        <span class="recruitment-text"> Are you an artist, willing to improve my tools? </span>

        <button class="recruitment-link" onclick={() => openUrl("https://discord.gg/qRTevFvHbx")}> Get in contact with me </button>
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
.advertisement-banner {
  background: transparent !important;
  position: relative;
  height: 250px;
  margin-bottom: $space-sm;
  border-radius: $radius-lg;
  overflow: hidden;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);

  &::after {
    content: "";
    position: absolute;
    inset: 0;
    z-index: 2;
    pointer-events: none;

    box-shadow:
      inset 0 1.5rem 1.5rem -1rem $color-background,
      inset 0 -1.5rem 1.5rem -1rem $color-background,
      inset 2.5rem 0 2.5rem -1rem $color-background,
      inset -2.5rem 0 2.5rem -1rem $color-background;
  }

  .banner-background {
    position: absolute;
    inset: 0;

    :global(.banner-image) {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }
  }

  .banner-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(to bottom, rgba(0, 0, 0, 0.3) 0%, rgba(0, 0, 0, 0.5) 100%);
  }

  .banner-content {
    position: relative;
    z-index: 1;
    height: 100%;

    display: flex;
    justify-content: center;
    align-items: center;

    .banner-actions {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 1.5rem;

      width: 100%;
      max-width: 1200px;
      padding: 0 2rem;

      justify-items: center;

      .banner-button {
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: center;
        gap: 0.75rem;

        max-width: fit-content;
        padding: 0.5rem 1rem;

        border: none;
        border-radius: 0.5rem;

        font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif !important;
        font-size: 1rem;
        font-weight: 900;
        text-decoration: none;

        cursor: pointer;

        transition: all 0.2s ease;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        backdrop-filter: blur(2px);

        :global(.button-image) {
          flex-shrink: 0;
          width: auto;
          height: 40px;
          object-fit: contain;
        }

        &:hover {
          transform: translateY(-2px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        }

        &:active {
          transform: translateY(0);
        }

        &.primary {
          background: #230f2c;
          color: var(--text);

          &:hover {
            background: rgba(#230f2c, 0.7);
          }
        }

        &.secondary {
          background: #db8b12;
          color: var(--text);

          &:hover {
            background: rgba(#db8b12, 0.7);
          }
        }

        &.kofi {
          background: #13c3a8;
          color: var(--text);
          border: 1px solid color-mix(in srgb, #13c3a8, 70%, transparent);

          &:hover {
            background: rgba(#13c3a8, 0.7);
          }
        }

        span {
          white-space: nowrap;
        }
      }
    }

    .artist-recruitment {
      position: absolute;
      bottom: $space-md;
      left: $space-3xl;

      display: flex;
      align-items: center;
      gap: 0.25rem;

      color: color-mix(in srgb, var(--text), 70%, transparent);
      font-size: 0.75rem;

      .recruitment-text {
        font-weight: 400;
      }

      .recruitment-link {
        padding: 0;

        background: none;
        border: none;

        color: var(--text);
        font-size: 0.75rem;
        font-weight: 500;
        text-decoration: underline;

        cursor: pointer;
        transition: color 0.2s ease;

        &:hover {
          color: $color-accent;
        }
      }
    }
  }
}
</style>
