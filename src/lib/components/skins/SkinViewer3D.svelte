<!-- @component
SkinViewer3D - 3D Minecraft skin viewer using skinview3d

Renders an interactive 3D preview of Minecraft skins with customizable
animations and models (slim/classic).

@prop {string} [skinUrl=''] - URL to the skin texture
@prop {number} [width=200] - Canvas width in pixels
@prop {number} [height=200] - Canvas height in pixels
@prop {'auto' | 'slim' | 'classic'} [model='auto'] - Skin model type
@prop {'idle' | 'walk' | 'run' | 'fly'} [animation='idle'] - Character animation

@example
```svelte
◄SkinViewer3D skinUrl="/skins/steve.png" width={300} height={400} animation="walk" /►
```
-->
<script lang="ts">
import { onMount, onDestroy } from "svelte";
import * as skinview3d from "skinview3d";

export let skinUrl: string = "";
export let width: number = 200;
export let height: number = 200;
export let model: "auto" | "slim" | "classic" = "auto";
export let animation: "idle" | "walk" | "run" | "fly" = "idle";

let canvas: HTMLCanvasElement;
let skinViewer: any = null;

onMount(() => {
  if (canvas && skinUrl) {
    initSkinViewer();
  }
});

onDestroy(() => {
  if (skinViewer) {
    skinViewer.dispose();
    skinViewer = null;
  }
});

function initSkinViewer() {
  if (!canvas || !skinUrl) return;

  try {
    // Create the skin viewer with basic options
    const options: any = {
      canvas,
      width,
      height,
      skin: skinUrl,
    };

    // Add model if not auto
    if (model === "classic") {
      options.model = "default"; // steve model
    } else if (model === "slim") {
      options.model = "slim"; // alex model
    }
    // For 'auto', let skinview3d auto-detect

    skinViewer = new skinview3d.SkinViewer(options);

    // Set animation based on available animations
    if (skinview3d.IdleAnimation) {
      switch (animation) {
        case "walk":
          if (skinview3d.WalkingAnimation) {
            skinViewer.animation = new skinview3d.WalkingAnimation();
          }
          break;
        case "run":
          if (skinview3d.RunningAnimation) {
            skinViewer.animation = new skinview3d.RunningAnimation();
          }
          break;
        case "fly":
          if (skinview3d.FlyingAnimation) {
            skinViewer.animation = new skinview3d.FlyingAnimation();
          }
          break;
        case "idle":
        default:
          skinViewer.animation = new skinview3d.IdleAnimation();
          break;
      }
    }

    // Set camera position for a nice view
    skinViewer.camera.position.x = 30;
    skinViewer.camera.position.y = 20;
    skinViewer.camera.position.z = 50;
    skinViewer.camera.lookAt(0, 10, 0);

    // Enable controls
    if (skinViewer.controls) {
      skinViewer.controls.enableRotate = true;
      skinViewer.controls.enableZoom = true;
      skinViewer.controls.enablePan = false;
    }
  } catch (error) {
    console.error("Failed to initialize SkinViewer3D:", error);
  }
}

// React to prop changes
$: if (skinViewer && skinUrl) {
  try {
    skinViewer.loadSkin(skinUrl);
  } catch (error) {
    console.error("Failed to load skin:", error);
  }
}

$: if (skinViewer && width && height) {
  skinViewer.width = width;
  skinViewer.height = height;
}

// React to animation changes
$: if (skinViewer && animation) {
  try {
    switch (animation) {
      case "walk":
        if (skinview3d.WalkingAnimation) {
          skinViewer.animation = new skinview3d.WalkingAnimation();
        }
        break;
      case "run":
        if (skinview3d.RunningAnimation) {
          skinViewer.animation = new skinview3d.RunningAnimation();
        }
        break;
      case "fly":
        if (skinview3d.FlyingAnimation) {
          skinViewer.animation = new skinview3d.FlyingAnimation();
        }
        break;
      case "idle":
      default:
        if (skinview3d.IdleAnimation) {
          skinViewer.animation = new skinview3d.IdleAnimation();
        }
        break;
    }
  } catch (error) {
    console.error("Failed to change animation:", error);
  }
}
</script>

<canvas
  bind:this={canvas}
  {width}
  {height}
  class="skin-viewer-canvas"
  style="width: {width}px; height: {height}px;"
></canvas>

<style>
.skin-viewer-canvas {
  border-radius: 8px;
  display: block;
}
</style>
