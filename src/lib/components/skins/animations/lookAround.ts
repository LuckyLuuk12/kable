import type { AnimatedPlayer, IdleGesture } from "./orchestrator";

type Rotation = {
  x: number;
  y: number;
  z: number;
};

function clamp(value: number, min = 0, max = 1): number {
  return Math.max(min, Math.min(max, value));
}

function lerp(from: number, to: number, t: number): number {
  return from + (to - from) * t;
}

function smoothstep(t: number): number {
  t = clamp(t);
  return t * t * (3 - 2 * t);
}

function interpolate(
  from: Rotation,
  to: Rotation,
  progress: number,
): Rotation {
  const t = smoothstep(progress);

  return {
    x: lerp(from.x, to.x, t),
    y: lerp(from.y, to.y, t),
    z: lerp(from.z, to.z, t),
  };
}

function applyRotation(
  joint: { rotation: Rotation },
  from: Rotation,
  to: Rotation,
  progress: number,
): void {
  const pose = interpolate(from, to, progress);

  joint.rotation.x = pose.x;
  joint.rotation.y = pose.y;
  joint.rotation.z = pose.z;
}

/**
 * A natural-looking look-around gesture.
 *
 * Can be triggered by ambient idle scheduling or a click.
 *
 * Timeline:
 *   0–15%: glance to the right
 *   15–35%: slowly look to the left
 *   35–55%: pause and look slightly upward
 *   55–75%: return to center
 *   75–100%: settle into the neutral pose
 */
export const lookAroundGesture: IdleGesture = {
  name: "look-around",
  duration: 5.5,
  weight: 1.4,
  triggers: ["idle", "click"],

  apply(player: AnimatedPlayer, progress: number): void {
    const skin = player.skin;
    if (!skin) return;

    const t = clamp(progress);

    const neutral: Rotation = { x: 0, y: 0, z: 0 };

    const rightGlance: Rotation = {
      x: -0.03,
      y: -0.45,
      z: 0.02,
    };

    const leftGlance: Rotation = {
      x: 0.02,
      y: 0.5,
      z: -0.015,
    };

    const lookUp: Rotation = {
      x: -0.28,
      y: 0.08,
      z: 0,
    };

    let from = neutral;
    let to = rightGlance;
    let localProgress = 0;

    if (t < 0.15) {
      // Center -> right.
      from = neutral;
      to = rightGlance;
      localProgress = t / 0.15;
    } else if (t < 0.35) {
      // Right -> left.
      from = rightGlance;
      to = leftGlance;
      localProgress = (t - 0.15) / 0.2;
    } else if (t < 0.55) {
      // Left -> upward glance.
      from = leftGlance;
      to = lookUp;
      localProgress = (t - 0.35) / 0.2;
    } else if (t < 0.75) {
      // Upward glance -> center.
      from = lookUp;
      to = neutral;
      localProgress = (t - 0.55) / 0.2;
    } else {
      // Settle at center.
      from = neutral;
      to = neutral;
      localProgress = (t - 0.75) / 0.25;
    }

    const headPose = interpolate(from, to, localProgress);

    // Keep the body nearly still, with a slight follow-through.
    const bodyPose: Rotation = {
      x: 0,
      y: headPose.y * 0.18,
      z: -headPose.y * 0.04,
    };

    applyRotation(skin.head, neutral, headPose, 1);
    applyRotation(skin.body, neutral, bodyPose, 1);
  },
};