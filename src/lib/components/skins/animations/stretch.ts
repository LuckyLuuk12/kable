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

/**
 * Smoothly interpolate between two poses.
 */
function easeInOut(t: number): number {
  t = clamp(t);
  return t * t * (3 - 2 * t);
}

function interpolateRotation(
  from: Rotation,
  to: Rotation,
  progress: number,
): Rotation {
  const t = easeInOut(progress);

  return {
    x: lerp(from.x, to.x, t),
    y: lerp(from.y, to.y, t),
    z: lerp(from.z, to.z, t),
  };
}

function applyRotation(
  target: { rotation: Rotation },
  from: Rotation,
  to: Rotation,
  progress: number,
): void {
  const rotation = interpolateRotation(from, to, progress);

  target.rotation.x = rotation.x;
  target.rotation.y = rotation.y;
  target.rotation.z = rotation.z;
}

/**
 * Full-body stretch gesture.
 *
 * Triggers: idle, click, hover.
 *
 * Timeline:
 *   0–25%: gradually raise both arms and lean back
 *   25–65%: hold the stretch with a subtle breathing motion
 *   65–100%: lower the arms and return upright
 *
 * Note: the current implementation uses a neutral rotation of zero.
 * If your viewer combines this gesture with another animation that
 * modifies the same joints, capture and restore a shared base pose.
 */
export const stretchGesture: IdleGesture = {
  name: "stretch",
  duration: 4.5,
  weight: 0.8,
  triggers: ["idle", "click", "hover"],

  apply(player: AnimatedPlayer, progress: number): void {
    const skin = player.skin;
    if (!skin) return;

    const t = clamp(progress);

    const neutral: Rotation = { x: 0, y: 0, z: 0 };

    // Both arms lift overhead and rotate slightly outward.
    const leftArmTarget: Rotation = {
      x: -Math.PI * 0.88,
      y: 0,
      z: 0.18,
    };

    const rightArmTarget: Rotation = {
      x: -Math.PI * 0.88,
      y: 0,
      z: -0.18,
    };

    // The torso leans back slightly; the head tilts up.
    const bodyTarget: Rotation = {
      x: -0.16,
      y: 0,
      z: 0,
    };

    const headTarget: Rotation = {
      x: -0.2,
      y: 0,
      z: 0,
    };

    let poseProgress: number;

    if (t < 0.25) {
      poseProgress = t / 0.25;
    } else if (t < 0.65) {
      poseProgress = 1;
    } else {
      poseProgress = 1 - (t - 0.65) / 0.35;
    }

    // Subtle motion while holding the stretch.
    const holdProgress =
      t >= 0.25 && t < 0.65
        ? Math.sin(((t - 0.25) / 0.4) * Math.PI)
        : 0;

    const bodyPose: Rotation = {
      x: bodyTarget.x - 0.025 * holdProgress,
      y: bodyTarget.y,
      z: bodyTarget.z,
    };

    const headPose: Rotation = {
      x: headTarget.x - 0.035 * holdProgress,
      y: headTarget.y,
      z: headTarget.z,
    };

    applyRotation(skin.leftArm, neutral, leftArmTarget, poseProgress);
    applyRotation(skin.rightArm, neutral, rightArmTarget, poseProgress);
    applyRotation(skin.body, neutral, bodyPose, poseProgress);
    applyRotation(skin.head, neutral, headPose, poseProgress);
  },
};