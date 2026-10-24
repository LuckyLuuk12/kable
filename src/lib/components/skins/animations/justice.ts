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

const DURATION = 7.2;
const PI = Math.PI;

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

function clamp01(value: number): number {
  return Math.max(0, Math.min(1, value));
}

function smootherStep(t: number): number {
  const x = clamp01(t);
  return x * x * x * (x * (x * 6 - 15) + 10);
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
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
 * Orange Justice inspired movement.
 *
 * Main choreography:
 *
 * 1. Arms swing outward.
 * 2. Arms cross in front of the torso.
 * 3. Knees sway inward and outward.
 * 4. The upper body shifts from side to side.
 * 5. Arm height alternates throughout the dance.
 *
 * The arms rotate forward while crossing to
 * reduce clipping through the torso.
 *
 * No elbow or knee joints are required.
 */
function createOrangeJusticePose(time: number): Pose {
  // Approximately 2.5 alternating arm positions per second.
  // One complete open-cross-open cycle takes 0.8 seconds.
  const beat = (time - 0.15) * 2.5;
  const phase = beat * PI;

  // +1 = arms spread outward.
  // -1 = arms crossed inward.
  const openClose = Math.cos(phase);

  // Normalized crossing amount.
  const crossed = (1 - openClose) * 0.5;

  // Slower side-to-side movement.
  const sway = Math.sin(phase * 0.5);

  // Quick movement between crossing positions.
  const quick = Math.sin(phase);

  // Slight forward/backward body pulse.
  const pulse = (1 - Math.cos(phase * 2)) * 0.5;

  return {
    ...NEUTRAL,

    // BODY
    // Forward lean and lateral rocking.
    bodyX:
      -0.10 +
      0.055 * pulse,

    bodyY:
      0.065 * sway,

    bodyZ:
      0.115 * sway +
      0.035 * quick,

    // HEAD
    // Small counter-movement against the torso.
    headX:
      0.035 -
      0.035 * pulse,

    headY:
      -0.10 * sway,

    headZ:
      -0.075 * sway,

    // LEFT ARM
    // Spread outward, then sweep across the chest.
    leftArmX:
      -0.35 -
      0.53 * crossed -
      0.12 * quick,

    leftArmY:
      0.08 +
      0.15 * sway,

    leftArmZ:
      0.16 +
      0.80 * openClose +
      0.18 * sway,

    // RIGHT ARM
    // Opposite movement creates the crossing action.
    rightArmX:
      -0.35 -
      0.53 * crossed +
      0.12 * quick,

    rightArmY:
      -0.08 +
      0.15 * sway,

    rightArmZ:
      -0.16 -
      0.80 * openClose +
      0.18 * sway,

    // LEFT LEG
    // Swing inward/outward with the arm rhythm.
    leftLegX:
      -0.10 -
      0.20 * sway +
      0.05 * crossed,

    leftLegY: 0,

    leftLegZ:
      0.14 * openClose +
      0.15 * quick,

    // RIGHT LEG
    // Opposite inward/outward movement.
    rightLegX:
      -0.10 +
      0.20 * sway +
      0.05 * crossed,

    rightLegY: 0,

    rightLegZ:
      -0.14 * openClose +
      0.15 * quick,
  };
}

interface TimedPose {
  at: number;
  pose: Pose;
}

/**
 * Final flourish:
 *
 * - Stop the repeated crossing pattern.
 * - Open the arms into a shrug.
 * - Bring both arms high.
 * - Return to a neutral position.
 *
 * All transitions are interpolated smoothly.
 */
function createFinishPose(time: number): Pose {
  const frames: TimedPose[] = [
    {
      at: 6.2,
      pose: createOrangeJusticePose(6.2),
    },
    {
      at: 6.43,
      pose: {
        ...NEUTRAL,

        bodyX: -0.12,
        bodyZ: 0.08,

        headX: -0.05,
        headZ: -0.04,

        leftArmX: -0.45,
        rightArmX: -0.45,

        leftArmZ: 1.30,
        rightArmZ: -1.30,

        leftLegX: -0.10,
        rightLegX: 0.10,

        leftLegZ: 0.12,
        rightLegZ: -0.12,
      },
    },
    {
      at: 6.66,
      pose: {
        ...NEUTRAL,

        bodyX: -0.12,

        headX: -0.12,

        leftArmX: -0.15,
        rightArmX: -0.15,

        leftArmZ: 2.80,
        rightArmZ: -2.80,

        leftLegZ: 0.13,
        rightLegZ: -0.13,
      },
    },
    {
      at: 6.82,
      pose: {
        ...NEUTRAL,

        bodyX: -0.04,

        leftArmX: -0.16,
        rightArmX: -0.16,

        leftArmZ: 1.45,
        rightArmZ: -1.45,

        leftLegZ: 0.06,
        rightLegZ: -0.06,
      },
    },
    {
      at: DURATION,
      pose: NEUTRAL,
    },
  ];

  for (let i = 0; i < frames.length - 1; i++) {
    const from = frames[i];
    const to = frames[i + 1];

    if (time <= to.at) {
      const progress =
        (time - from.at) / (to.at - from.at);

      return interpolatePose(
        from.pose,
        to.pose,
        progress,
      );
    }
  }

  return NEUTRAL;
}

/**
 * Minecraft Orange Justice-inspired dance.
 *
 * Duration: 7.2 seconds
 * Trigger: O
 *
 * Uses the standard Minecraft skin parts.
 * No additional bones or joints needed.
 */
export const justiceGesture: IdleGesture = {
  name: "dance",
  duration: DURATION,
  weight: 1.5,
  triggers: ["keybind"],
  keybind: "o",

  apply(player: AnimatedPlayer, progress: number) {
    const p = clamp01(progress);
    const time = p * DURATION;

    let pose: Pose;

    // Main repeating Orange Justice movement.
    if (time < 6.2) {
      pose = createOrangeJusticePose(time);
    } else {
      // Final arm flourish and neutral return.
      pose = createFinishPose(time);
    }

    // Smooth introduction.
    // Keeps the first frame exactly neutral.
    if (time < 0.28) {
      pose = interpolatePose(
        NEUTRAL,
        pose,
        time / 0.28,
      );
    }

    const skin = player.skin;

    // BODY
    skin.body.rotation.x = pose.bodyX;
    skin.body.rotation.y = pose.bodyY;
    skin.body.rotation.z = pose.bodyZ;

    // HEAD
    skin.head.rotation.x = pose.headX;
    skin.head.rotation.y = pose.headY;
    skin.head.rotation.z = pose.headZ;

    // LEFT ARM
    skin.leftArm.rotation.x = pose.leftArmX;
    skin.leftArm.rotation.y = pose.leftArmY;
    skin.leftArm.rotation.z = pose.leftArmZ;

    // RIGHT ARM
    skin.rightArm.rotation.x = pose.rightArmX;
    skin.rightArm.rotation.y = pose.rightArmY;
    skin.rightArm.rotation.z = pose.rightArmZ;

    // LEFT LEG
    skin.leftLeg.rotation.x = pose.leftLegX;
    skin.leftLeg.rotation.y = pose.leftLegY;
    skin.leftLeg.rotation.z = pose.leftLegZ;

    // RIGHT LEG
    skin.rightLeg.rotation.x = pose.rightLegX;
    skin.rightLeg.rotation.y = pose.rightLegY;
    skin.rightLeg.rotation.z = pose.rightLegZ;
  },
};