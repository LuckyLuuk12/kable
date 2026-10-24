import type { AnimatedPlayer, IdleGesture } from "./orchestrator";

/**
 * Smoothly interpolate between two values.
 */
function lerp(from: number, to: number, t: number): number {
  return from + (to - from) * t;
}

/**
 * Smoothstep easing: gradual start and finish.
 */
function smoothstep(t: number): number {
  const clamped = Math.max(0, Math.min(1, t));
  return clamped * clamped * (3 - 2 * clamped);
}

/**
 * Interpolate a joint from its starting rotation to its target rotation.
 */
function interpolatePose(
  from: { x: number; y: number; z: number },
  to: { x: number; y: number; z: number },
  progress: number,
) {
  const t = smoothstep(progress);

  return {
    x: lerp(from.x, to.x, t),
    y: lerp(from.y, to.y, t),
    z: lerp(from.z, to.z, t),
  };
}

/**
 * Pointing gesture.
 *
 * Trigger: drag only.
 *
 * Timeline:
 *   0–25%: raise the arm and turn the head
 *   25–70%: hold the pointing pose
 *   70–100%: return to the neutral pose
 *
 * The pose is expressed as absolute joint rotations, not incremental
 * rotations, so repeated animation frames do not accumulate rotation.
 */
export const pointGesture: IdleGesture = {
  name: "point",
  duration: 3.2,
  weight: 1,
  triggers: ["drag", "hover"],

  apply(player: AnimatedPlayer, progress: number): void {
    const skin = player.skin;
    if (!skin) return;

    const t = Math.max(0, Math.min(1, progress));

    const neutral = {
      rightArm: { x: 0, y: 0, z: 0 },
      head: { x: 0, y: 0, z: 0 },
    };

    const pointPose = {
      rightArm: {
        x: -Math.PI / 2,
        y: 0,
        z: -0.12,
      },
      head: {
        x: -0.08,
        y: -0.35,
        z: 0,
      },
    };

    // Raise, hold, and lower the arm.
    let poseProgress: number;

    if (t < 0.25) {
      poseProgress = t / 0.25;
    } else if (t < 0.7) {
      poseProgress = 1;
    } else {
      poseProgress = 1 - (t - 0.7) / 0.3;
    }

    const armPose = interpolatePose(
      neutral.rightArm,
      pointPose.rightArm,
      poseProgress,
    );

    const headPose = interpolatePose(
      neutral.head,
      pointPose.head,
      poseProgress,
    );

    skin.rightArm.rotation.x = armPose.x;
    skin.rightArm.rotation.y = armPose.y;
    skin.rightArm.rotation.z = armPose.z;

    skin.head.rotation.x = headPose.x;
    skin.head.rotation.y = headPose.y;
    skin.head.rotation.z = headPose.z;
  },
};