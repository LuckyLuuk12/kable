import type { AnimatedPlayer, IdleGesture } from "./orchestrator";

interface Pose {
  bodyX: number;
  bodyY: number;
  bodyZ: number;

  headX: number;
  headY: number;
  headZ: number;

  leftArmX: number;
  leftArmY: number;
  leftArmZ: number;

  rightArmX: number;
  rightArmY: number;
  rightArmZ: number;

  leftLegX: number;
  leftLegY: number;
  leftLegZ: number;

  rightLegX: number;
  rightLegY: number;
  rightLegZ: number;
}

const TAU = Math.PI * 2;

const NEUTRAL: Pose = {
  bodyX: 0,
  bodyY: 0,
  bodyZ: 0,

  headX: 0,
  headY: 0,
  headZ: 0,

  leftArmX: 0,
  leftArmY: 0,
  leftArmZ: 0,

  rightArmX: 0,
  rightArmY: 0,
  rightArmZ: 0,

  leftLegX: 0,
  leftLegY: 0,
  leftLegZ: 0,

  rightLegX: 0,
  rightLegY: 0,
  rightLegZ: 0,
};

interface Keyframe {
  at: number;
  pose: Partial<Pose>;
}

/**
 * Smooth interpolation with zero velocity at both endpoints.
 * This avoids abrupt changes when entering or leaving a pose.
 */
function smootherStep(t: number): number {
  const x = Math.max(0, Math.min(1, t));
  return x * x * x * (x * (x * 6 - 15) + 10);
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

function wave(t: number, cycles: number, phase = 0): number {
  return Math.sin(t * TAU * cycles + phase);
}

function interpolatePose(
  from: Pose,
  to: Pose,
  progress: number,
): Pose {
  const t = smootherStep(progress);
  const result = {} as Pose;

  for (const key of Object.keys(NEUTRAL) as (keyof Pose)[]) {
    result[key] = lerp(from[key], to[key], t);
  }

  return result;
}

/**
 * A playful dance with:
 * - A gentle side-to-side groove
 * - Alternating knee lifts
 * - A disco-point sequence
 * - A short shoulder shimmy
 * - A celebratory finish
 *
 * All major poses are interpolated, and the final pose is neutral.
 */
export const danceGesture: IdleGesture = {
  name: "dance",
  duration: 7.2,
  weight: 1.5,
  triggers: ["keybind"],
  keybind: "d",

  apply(player: AnimatedPlayer, progress: number) {
    const p = Math.max(0, Math.min(1, progress));
    const time = p * 7.2;

    const keyframes: Keyframe[] = [
      {
        at: 0,
        pose: NEUTRAL,
      },

      // Ease into the groove.
      {
        at: 0.08,
        pose: {
          bodyZ: -0.12,
          bodyY: -0.08,
          headZ: 0.08,
          leftArmX: -0.12,
          rightArmX: 0.12,
          leftArmZ: 0.08,
          rightArmZ: -0.08,
        },
      },

      // Left knee up, opposite arm swinging forward.
      {
        at: 0.19,
        pose: {
          bodyX: -0.1,
          bodyZ: -0.16,
          bodyY: -0.1,
          headZ: 0.1,
          leftArmX: -0.5,
          rightArmX: 0.4,
          leftArmZ: 0.18,
          rightArmZ: -0.12,
          leftLegX: -0.58,
          rightLegX: 0.16,
        },
      },

      // Land and shift weight.
      {
        at: 0.28,
        pose: {
          bodyX: 0.04,
          bodyZ: 0.13,
          bodyY: 0.04,
          headZ: -0.08,
          leftArmX: 0.15,
          rightArmX: -0.35,
          leftArmZ: 0.05,
          rightArmZ: -0.1,
          leftLegX: 0.08,
          rightLegX: -0.08,
        },
      },

      // Right knee up.
      {
        at: 0.39,
        pose: {
          bodyX: -0.1,
          bodyZ: 0.16,
          bodyY: -0.1,
          headZ: -0.1,
          leftArmX: 0.4,
          rightArmX: -0.5,
          leftArmZ: 0.12,
          rightArmZ: -0.18,
          leftLegX: 0.16,
          rightLegX: -0.58,
        },
      },

      // Settle into the next move.
      {
        at: 0.48,
        pose: {
          bodyX: 0.02,
          bodyZ: -0.12,
          bodyY: 0.04,
          headZ: 0.08,
          leftArmX: -0.2,
          rightArmX: 0.2,
          leftArmZ: 0.05,
          rightArmZ: -0.05,
          leftLegX: -0.08,
          rightLegX: 0.08,
        },
      },

      // Disco point to one side.
      {
        at: 0.59,
        pose: {
          bodyX: -0.06,
          bodyY: -0.04,
          bodyZ: -0.2,
          headX: -0.08,
          headY: -0.12,
          headZ: 0.14,
          leftArmX: -0.15,
          rightArmX: -0.15,
          leftArmZ: 0.85,
          rightArmZ: -0.35,
          leftLegX: -0.16,
          rightLegX: 0.1,
        },
      },

      // Flow through center rather than snapping to the other side.
      {
        at: 0.66,
        pose: {
          bodyZ: 0.03,
          headY: 0.04,
          headZ: 0,
          leftArmZ: 0.25,
          rightArmZ: -0.1,
          leftLegX: 0.04,
          rightLegX: -0.04,
        },
      },

      // Disco point to the other side.
      {
        at: 0.73,
        pose: {
          bodyX: -0.06,
          bodyY: -0.04,
          bodyZ: 0.2,
          headX: -0.08,
          headY: 0.12,
          headZ: -0.14,
          leftArmX: -0.15,
          rightArmX: -0.15,
          leftArmZ: 0.35,
          rightArmZ: -0.85,
          leftLegX: 0.1,
          rightLegX: -0.16,
        },
      },

      // Relax the arms into a shoulder shimmy.
      {
        at: 0.81,
        pose: {
          bodyX: 0.02,
          bodyY: 0,
          bodyZ: 0,
          headX: 0,
          headY: 0,
          headZ: 0,
          leftArmX: -0.1,
          rightArmX: -0.1,
          leftArmZ: 0.15,
          rightArmZ: -0.15,
          leftLegX: -0.05,
          rightLegX: 0.05,
        },
      },

      // Victory pose, arms raised without forcing them through the torso.
      {
        at: 0.9,
        pose: {
          bodyX: -0.08,
          bodyY: -0.06,
          bodyZ: 0,
          headX: -0.12,
          headY: 0,
          headZ: 0,
          leftArmX: -0.25,
          rightArmX: -0.25,
          leftArmZ: 0.95,
          rightArmZ: -0.95,
          leftLegX: -0.12,
          rightLegX: 0.12,
        },
      },

      // Lower the arms smoothly before the animation ends.
      {
        at: 0.96,
        pose: {
          bodyX: 0,
          bodyY: 0,
          bodyZ: 0,
          headX: 0,
          headY: 0,
          headZ: 0,
          leftArmX: 0,
          rightArmX: 0,
          leftArmZ: 0.08,
          rightArmZ: -0.08,
          leftLegX: 0,
          rightLegX: 0,
        },
      },

      {
        at: 1,
        pose: NEUTRAL,
      },
    ];

    let segment = 0;

    while (
      segment < keyframes.length - 2 &&
      p > keyframes[segment + 1].at
    ) {
      segment++;
    }

    const fromFrame = keyframes[segment];
    const toFrame = keyframes[segment + 1];

    const segmentProgress =
      (p - fromFrame.at) / (toFrame.at - fromFrame.at);

    const fromPose: Pose = {
      ...NEUTRAL,
      ...fromFrame.pose,
    };

    const toPose: Pose = {
      ...NEUTRAL,
      ...toFrame.pose,
    };

    const pose = interpolatePose(
      fromPose,
      toPose,
      segmentProgress,
    );

    // Continuous groove layered over the interpolated poses.
    // Fade it in and out so the beginning and ending remain neutral.
    const envelope = smootherStep(Math.min(p / 0.08, 1)) *
      (1 - smootherStep(Math.max(0, (p - 0.9) / 0.1)));

    const groove = wave(time / 7.2, 1.7);
    const bounce = (1 - Math.cos(time * TAU * 1.7 / 2)) / 2;

    pose.bodyY += -0.045 * bounce * envelope;
    pose.bodyZ += 0.035 * groove * envelope;
    pose.headY += 0.045 * wave(time / 7.2, 0.85, 0.7) * envelope;
    pose.headZ += 0.025 * wave(time / 7.2, 1.7, 0.5) * envelope;

    // A restrained shimmy keeps the character moving between poses.
    const shimmy = wave(time / 7.2, 5.5) * envelope;

    pose.leftArmX += shimmy * 0.07;
    pose.rightArmX -= shimmy * 0.07;
    pose.leftArmZ += shimmy * 0.035;
    pose.rightArmZ -= shimmy * 0.035;

    const skin = player.skin;

    skin.body.rotation.x = pose.bodyX;
    skin.body.rotation.y = pose.bodyY;
    skin.body.rotation.z = pose.bodyZ;

    skin.head.rotation.x = pose.headX;
    skin.head.rotation.y = pose.headY;
    skin.head.rotation.z = pose.headZ;

    skin.leftArm.rotation.x = pose.leftArmX;
    skin.leftArm.rotation.y = pose.leftArmY;
    skin.leftArm.rotation.z = pose.leftArmZ;

    skin.rightArm.rotation.x = pose.rightArmX;
    skin.rightArm.rotation.y = pose.rightArmY;
    skin.rightArm.rotation.z = pose.rightArmZ;

    skin.leftLeg.rotation.x = pose.leftLegX;
    skin.leftLeg.rotation.y = pose.leftLegY;
    skin.leftLeg.rotation.z = pose.leftLegZ;

    skin.rightLeg.rotation.x = pose.rightLegX;
    skin.rightLeg.rotation.y = pose.rightLegY;
    skin.rightLeg.rotation.z = pose.rightLegZ;
  },
};
