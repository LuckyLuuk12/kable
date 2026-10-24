import type { AnimatedPlayer, IdleGesture } from "./orchestrator";

interface Pose {
  bodyX: number; bodyY: number; bodyZ: number;
  headX: number; headY: number; headZ: number;
  leftArmX: number; leftArmY: number; leftArmZ: number;
  rightArmX: number; rightArmY: number; rightArmZ: number;
  leftLegX: number; leftLegY: number; leftLegZ: number;
  rightLegX: number; rightLegY: number; rightLegZ: number;
  crouch: number;
}

interface Keyframe {
  at: number;
  pose: Partial<Pose>;
}

const TAU = Math.PI * 2;
const DURATION = 8.4;
const CROUCH_DROP = 2.6;

const NEUTRAL: Pose = {
  bodyX: 0, bodyY: 0, bodyZ: 0,
  headX: 0, headY: 0, headZ: 0,
  leftArmX: 0, leftArmY: 0, leftArmZ: 0,
  rightArmX: 0, rightArmY: 0, rightArmZ: 0,
  leftLegX: 0, leftLegY: 0, leftLegZ: 0,
  rightLegX: 0, rightLegY: 0, rightLegZ: 0,
  crouch: 0,
};

const BASE: Pose = {
  ...NEUTRAL,
  bodyX: -0.16,
  headX: 0.08,
  leftLegZ: 0.15,
  rightLegZ: -0.15,
};

function smootherStep(t: number): number {
  const x = Math.max(0, Math.min(1, t));
  return x * x * x * (x * (x * 6 - 15) + 10);
}

function interpolatePose(
  from: Pose,
  to: Pose,
  progress: number,
): Pose {
  const t = smootherStep(progress);
  const result = {} as Pose;

  for (const key of Object.keys(NEUTRAL) as (keyof Pose)[]) {
    result[key] = from[key] + (to[key] - from[key]) * t;
  }

  return result;
}

/**
 * Aggressive Infectious-inspired industrial dance.
 *
 * Main features:
 * - Large overhead arm swings.
 * - Alternating diagonal strikes.
 * - Deep simulated crouches.
 * - Fast mechanical upper-body movement.
 * - Rapid alternating footwork.
 *
 * All poses use the existing six-part Minecraft rig.
 */
const KEYFRAMES: Keyframe[] = [
  {
    at: 0,
    pose: NEUTRAL,
  },

  // Enter aggressive industrial stance.
  {
    at: 0.22,
    pose: {
      bodyX: -0.24,
      headX: 0.12,
      leftArmX: -0.75,
      rightArmX: -0.75,
      leftArmZ: 0.50,
      rightArmZ: -0.50,
      leftLegZ: 0.22,
      rightLegZ: -0.22,
      crouch: 0.20,
    },
  },

  // Right arm shoots above the head.
  {
    at: 0.53,
    pose: {
      bodyX: -0.30,
      bodyY: 0.18,
      bodyZ: 0.13,
      headY: -0.18,
      rightArmX: -0.35,
      rightArmZ: -2.55,
      leftArmX: -1.15,
      leftArmZ: -0.40,
      leftLegX: -0.35,
      rightLegX: 0.25,
      crouch: 0.35,
    },
  },

  // Switch: left arm shoots above the head.
  {
    at: 0.86,
    pose: {
      bodyX: -0.30,
      bodyY: -0.18,
      bodyZ: -0.13,
      headY: 0.18,
      leftArmX: -0.35,
      leftArmZ: 2.55,
      rightArmX: -1.15,
      rightArmZ: 0.40,
      leftLegX: 0.25,
      rightLegX: -0.35,
      crouch: 0.35,
    },
  },

  // DEEP DIP: arms slash downward.
  {
    at: 1.16,
    pose: {
      bodyX: -0.48,
      headX: 0.25,
      leftArmX: -1.35,
      rightArmX: -1.35,
      leftArmZ: -0.45,
      rightArmZ: 0.45,
      leftLegX: -0.85,
      rightLegX: 0.85,
      leftLegZ: 0.36,
      rightLegZ: -0.36,
      crouch: 1.0,
    },
  },

  // Explode upward with both arms overhead.
  {
    at: 1.49,
    pose: {
      bodyX: -0.12,
      headX: -0.20,
      leftArmX: -0.30,
      rightArmX: -0.30,
      leftArmZ: 2.80,
      rightArmZ: -2.80,
      leftLegZ: 0.18,
      rightLegZ: -0.18,
      crouch: 0.05,
    },
  },

  // Right overhead, left sweeping across.
  {
    at: 1.82,
    pose: {
      bodyX: -0.28,
      bodyY: 0.22,
      bodyZ: 0.15,
      rightArmX: -0.25,
      rightArmZ: -2.65,
      leftArmX: -1.20,
      leftArmZ: -0.65,
      leftLegX: -0.30,
      rightLegX: 0.25,
      crouch: 0.28,
    },
  },

  // Second deep crouch: arms thrown outward.
  {
    at: 2.16,
    pose: {
      bodyX: -0.52,
      bodyY: -0.12,
      bodyZ: -0.10,
      headX: 0.30,
      leftArmX: -0.80,
      rightArmX: -0.80,
      leftArmZ: 1.55,
      rightArmZ: -1.55,
      leftLegX: 0.90,
      rightLegX: -0.90,
      leftLegZ: 0.40,
      rightLegZ: -0.40,
      crouch: 0.95,
    },
  },

  // Left arm swings high across the head.
  {
    at: 2.47,
    pose: {
      bodyX: -0.22,
      bodyY: -0.22,
      headY: 0.15,
      leftArmX: -0.40,
      leftArmZ: 2.65,
      rightArmX: -1.10,
      rightArmZ: 0.70,
      leftLegX: -0.25,
      rightLegX: 0.20,
      crouch: 0.18,
    },
  },

  // Right arm swings high across the head.
  {
    at: 2.79,
    pose: {
      bodyX: -0.22,
      bodyY: 0.22,
      headY: -0.15,
      leftArmX: -1.10,
      leftArmZ: -0.70,
      rightArmX: -0.40,
      rightArmZ: -2.65,
      leftLegX: 0.20,
      rightLegX: -0.25,
      crouch: 0.18,
    },
  },

  // Both arms overhead again.
  {
    at: 3.10,
    pose: {
      bodyX: -0.20,
      headX: -0.18,
      leftArmX: -0.20,
      rightArmX: -0.20,
      leftArmZ: 2.95,
      rightArmZ: -2.95,
      crouch: 0.10,
    },
  },

  // DROP: fast low crossed-arm strike.
  {
    at: 3.40,
    pose: {
      bodyX: -0.55,
      headX: 0.35,
      leftArmX: -1.40,
      rightArmX: -1.40,
      leftArmZ: -0.70,
      rightArmZ: 0.70,
      leftLegX: -0.95,
      rightLegX: 0.95,
      leftLegZ: 0.40,
      rightLegZ: -0.40,
      crouch: 1.0,
    },
  },

  // Violent upward left-arm sweep.
  {
    at: 3.72,
    pose: {
      bodyX: -0.28,
      bodyY: -0.25,
      bodyZ: -0.16,
      leftArmX: -0.28,
      leftArmZ: 2.75,
      rightArmX: -1.20,
      rightArmZ: 0.40,
      leftLegX: 0.25,
      rightLegX: -0.25,
      crouch: 0.25,
    },
  },

  // Violent upward right-arm sweep.
  {
    at: 4.04,
    pose: {
      bodyX: -0.28,
      bodyY: 0.25,
      bodyZ: 0.16,
      rightArmX: -0.28,
      rightArmZ: -2.75,
      leftArmX: -1.20,
      leftArmZ: -0.40,
      leftLegX: -0.25,
      rightLegX: 0.25,
      crouch: 0.25,
    },
  },

  // Large overhead windmill-like pose.
  {
    at: 4.35,
    pose: {
      bodyX: -0.20,
      headX: -0.15,
      leftArmX: -0.35,
      leftArmZ: 2.85,
      rightArmX: -0.50,
      rightArmZ: -2.25,
      crouch: 0.08,
    },
  },

  // Another hard drop.
  {
    at: 4.66,
    pose: {
      bodyX: -0.50,
      headX: 0.30,
      leftArmX: -1.35,
      leftArmZ: -0.60,
      rightArmX: -1.35,
      rightArmZ: 0.60,
      leftLegX: 0.88,
      rightLegX: -0.88,
      leftLegZ: 0.40,
      rightLegZ: -0.40,
      crouch: 0.95,
    },
  },

  // Left overhead, right downward.
  {
    at: 4.98,
    pose: {
      bodyX: -0.24,
      bodyY: -0.23,
      headZ: 0.13,
      leftArmX: -0.25,
      leftArmZ: 2.70,
      rightArmX: -1.10,
      rightArmZ: 0.20,
      crouch: 0.20,
    },
  },

  // Right overhead, left downward.
  {
    at: 5.30,
    pose: {
      bodyX: -0.24,
      bodyY: 0.23,
      headZ: -0.13,
      leftArmX: -1.10,
      leftArmZ: -0.20,
      rightArmX: -0.25,
      rightArmZ: -2.70,
      crouch: 0.20,
    },
  },

  // Both arms spread wide.
  {
    at: 5.62,
    pose: {
      bodyX: -0.30,
      leftArmX: -0.60,
      rightArmX: -0.60,
      leftArmZ: 1.65,
      rightArmZ: -1.65,
      leftLegZ: 0.28,
      rightLegZ: -0.28,
      crouch: 0.32,
    },
  },

  // Fast drop with arms crossing.
  {
    at: 5.94,
    pose: {
      bodyX: -0.55,
      headX: 0.35,
      leftArmX: -1.30,
      rightArmX: -1.30,
      leftArmZ: -0.75,
      rightArmZ: 0.75,
      leftLegX: -0.90,
      rightLegX: 0.90,
      leftLegZ: 0.38,
      rightLegZ: -0.38,
      crouch: 1.0,
    },
  },

  // Snap up: left overhead.
  {
    at: 6.26,
    pose: {
      bodyX: -0.18,
      bodyY: -0.20,
      leftArmX: -0.25,
      leftArmZ: 2.85,
      rightArmX: -1.10,
      rightArmZ: 0.50,
      crouch: 0.08,
    },
  },

  // Snap to the other side.
  {
    at: 6.58,
    pose: {
      bodyX: -0.18,
      bodyY: 0.20,
      rightArmX: -0.25,
      rightArmZ: -2.85,
      leftArmX: -1.10,
      leftArmZ: -0.50,
      crouch: 0.08,
    },
  },

  // Final aggressive crouch.
  {
    at: 6.90,
    pose: {
      bodyX: -0.52,
      headX: 0.32,
      leftArmX: -1.25,
      rightArmX: -1.25,
      leftArmZ: -0.65,
      rightArmZ: 0.65,
      leftLegX: 0.88,
      rightLegX: -0.88,
      leftLegZ: 0.38,
      rightLegZ: -0.38,
      crouch: 1.0,
    },
  },

  // Explosive overhead finish.
  {
    at: 7.23,
    pose: {
      bodyX: -0.15,
      headX: -0.20,
      leftArmX: -0.22,
      rightArmX: -0.22,
      leftArmZ: 2.90,
      rightArmZ: -2.90,
      crouch: 0.05,
    },
  },

  // Arms swing back out.
  {
    at: 7.62,
    pose: {
      bodyX: -0.10,
      leftArmX: -0.40,
      rightArmX: -0.40,
      leftArmZ: 1.10,
      rightArmZ: -1.10,
      crouch: 0.05,
    },
  },

  // Return to neutral.
  {
    at: DURATION,
    pose: NEUTRAL,
  },
];

function getPose(time: number): Pose {
  let segment = 0;

  while (
    segment < KEYFRAMES.length - 2 &&
    time > KEYFRAMES[segment + 1].at
  ) {
    segment++;
  }

  const from = KEYFRAMES[segment];
  const to = KEYFRAMES[segment + 1];

  const progress =
    (time - from.at) / (to.at - from.at);

  return interpolatePose(
    { ...BASE, ...from.pose },
    { ...BASE, ...to.pose },
    progress,
  );
}

/**
 * Tracks the height offset added by this animation.
 * Delta updates avoid accumulating a new offset on
 * every animation frame.
 */
const appliedOffsets = new WeakMap<object, number>();

function setCrouchHeight(
  skin: AnimatedPlayer["skin"],
  offset: number,
): void {
  const movable = skin as typeof skin & {
    position?: { y: number };
  };

  if (!movable.position) return;

  const previous = appliedOffsets.get(skin) ?? 0;

  // Remove our previous offset and apply the new one.
  movable.position.y += previous - offset;

  appliedOffsets.set(skin, offset);
}

/**
 * Call this when the orchestrator interrupts
 * Infectious before its normal ending.
 */
export function resetInfectiousHeight(
  player: AnimatedPlayer,
): void {
  setCrouchHeight(player.skin, 0);
}

export const infectiousGesture: IdleGesture = {
  name: "infectious",
  duration: DURATION,
  weight: 1.5,
  triggers: ["keybind"],
  keybind: "i",

  apply(player: AnimatedPlayer, progress: number) {
    const p = Math.max(0, Math.min(1, progress));
    const time = p * DURATION;

    const pose = getPose(time);

    const envelope =
      smootherStep(time / 0.25) *
      (1 - smootherStep((time - 7.85) / 0.55));

    // Rapid alternating foot strikes.
    const stomp = Math.sin(time * TAU * 2.6);

    pose.leftLegX +=
      -0.15 * Math.max(0, stomp) * envelope;

    pose.rightLegX +=
      -0.15 * Math.max(0, -stomp) * envelope;

    // Additional rhythmic ducking.
    const pulse =
      (1 - Math.cos(time * TAU * 2.6)) * 0.5;

    pose.crouch = Math.min(
      1,
      pose.crouch + pulse * 0.18 * envelope,
    );

    // Headbanging.
    pose.headX +=
      0.14 *
      Math.sin(time * TAU * 2.6 + 0.5) *
      envelope;

    // Rapid torso movement.
    pose.bodyY +=
      0.09 *
      Math.sin(time * TAU * 1.3) *
      envelope;

    pose.bodyZ +=
      0.055 *
      Math.sin(time * TAU * 2.6) *
      envelope;

    // Mechanical arm shake between major strikes.
    const armShake =
      Math.sin(time * TAU * 5.2) * envelope;

    pose.leftArmX += armShake * 0.075;
    pose.rightArmX -= armShake * 0.075;

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

    // Lower the entire model during deep crouches.
    setCrouchHeight(
      skin,
      CROUCH_DROP * pose.crouch,
    );
  },
};