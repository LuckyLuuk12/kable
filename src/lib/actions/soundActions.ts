import { soundService } from "$lib/services/SoundService";

/**
 * Action to add sound effects to interactive elements (buttons, links, etc.)
 * Works on any HTMLElement that can receive click and mouseenter events.
 *
 * @example
 * // Basic usage
 * <button use:clickSound>Click me</button>
 * <a href="/page" use:clickSound>Link</a>
 *
 * @example
 * // Custom sounds
 * <button use:clickSound={{ click: 'launch', hover: 'hover' }}>Launch</button>
 * <a href="/settings" use:clickSound={{ click: 'success' }}>Settings</a>
 */
export function clickSound(
  node: HTMLElement,
  options: { click?: string; hover?: string } = {},
) {
  const clickSound = options.click || "click";
  const hoverSound = options.hover || "hover";

  const handleClick = () => {
    soundService.playSound(clickSound);
  };

  const handleMouseEnter = () => {
    soundService.playSound(hoverSound, { volume: 0.3 });
  };

  node.addEventListener("click", handleClick);
  node.addEventListener("mouseenter", handleMouseEnter);

  return {
    destroy() {
      node.removeEventListener("click", handleClick);
      node.removeEventListener("mouseenter", handleMouseEnter);
    },
    update(newOptions: { click?: string; hover?: string } = {}) {
      // Update internal references if options change
      const newClickSound = newOptions.click || "click";
      const newHoverSound = newOptions.hover || "hover";

      // Remove old listeners
      node.removeEventListener("click", handleClick);
      node.removeEventListener("mouseenter", handleMouseEnter);

      // Create new handlers with updated sounds
      const newHandleClick = () => {
        soundService.playSound(newClickSound);
      };

      const newHandleMouseEnter = () => {
        soundService.playSound(newHoverSound, { volume: 0.3 });
      };

      // Add new listeners
      node.addEventListener("click", newHandleClick);
      node.addEventListener("mouseenter", newHandleMouseEnter);
    },
  };
}

/**
 * Action to play success sound on click
 * @example
 * <button use:successSound>Save</button>
 * <a href="/save" use:successSound>Save Link</a>
 */
export function successSound(node: HTMLElement) {
  return clickSound(node, { click: "success" });
}

/**
 * Action to play error sound on click
 * @example
 * <button use:errorSound>Delete</button>
 * <a href="/delete" use:errorSound>Delete Link</a>
 */
export function errorSound(node: HTMLElement) {
  return clickSound(node, { click: "error" });
}

/**
 * Action to play notification sound on click
 * @example
 * <button use:notificationSound>Notify</button>
 * <a href="/notifications" use:notificationSound>Notifications</a>
 */
export function notificationSound(node: HTMLElement) {
  return clickSound(node, { click: "notification" });
}

/**
 * Action to play launch sound on click
 * @example
 * <button use:launchSound>Launch Game</button>
 * <a href="/launch" use:launchSound>Launch Link</a>
 */
export function launchSound(node: HTMLElement) {
  return clickSound(node, { click: "launch" });
}

// Backward compatibility alias
export const buttonSound = clickSound;

/**
 * Helper to play sound on specific events
 */
export function playOnEvent(soundKey: string) {
  soundService.playSound(soundKey);
}

/**
 * Helper to play music
 */
export function playMusic(playlistKey: string) {
  soundService.playBackgroundMusic(playlistKey);
}

/**
 * Helper to stop music
 */
export function stopMusic() {
  soundService.stopBackgroundMusic();
}
