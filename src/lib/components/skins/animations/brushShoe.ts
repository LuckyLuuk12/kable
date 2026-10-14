import type { AnimatedPlayer, IdleGesture } from "./orchestrator";

type Vector3 = {
  x: number;
  y: number;
  z: number;
};

type Transform = {
  rotation: Vector3;
  position: Vector3;
};

type Pose = {
  body: Transform;
  head: Transform;
  rightArm: Transform;
  leftArm: Transform;
  rightLeg: Transform;
  leftLeg: Transform;
};

const ZERO: Vector3 = { x: 0, y: 0, z: 0 };

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

function interpolateVector(
  from: Vector3,
  to: Vector3,
  progress: number,
): Vector3 {
  const t = smoothstep(progress);

  return {
    x: lerp(from.x, to.x, t),
    y: lerp(from.y, to.y, t),
    z: lerp(from.z, to.z, t),
  };
}

function interpolateTransform(
  from: Transform,
  to: Transform,
  progress: number,
): Transform {
  return {
    rotation: interpolateVector(from.rotation, to.rotation, progress),
    position: interpolateVector(from.position, to.position, progress),
  };
}

function transform(
  rotation: Partial<Vector3> = {},
  position: Partial<Vector3> = {},
): Transform {
  return {
    rotation: { ...ZERO, ...rotation },
    position: { ...ZERO, ...position },
  };
}

function interpolatePose(
  from: Pose,
  to: Pose,
  progress: number,
): Pose {
  return {
    body: interpolateTransform(from.body, to.body, progress),
    head: interpolateTransform(from.head, to.head, progress),
    rightArm: interpolateTransform(from.rightArm, to.rightArm, progress),
    leftArm: interpolateTransform(from.leftArm, to.leftArm, progress),
    rightLeg: interpolateTransform(from.rightLeg, to.rightLeg, progress),
    leftLeg: interpolateTransform(from.leftLeg, to.leftLeg, progress),
  };
}

function applyTransform(
  joint: {
    rotation: Vector3;
    position: Vector3;
  },
  base: Transform,
  offset: Transform,
): void {
  joint.rotation.x = base.rotation.x + offset.rotation.x;
  joint.rotation.y = base.rotation.y + offset.rotation.y;
  joint.rotation.z = base.rotation.z + offset.rotation.z;

  joint.position.x = base.position.x + offset.position.x;
  joint.position.y = base.position.y + offset.position.y;
  joint.position.z = base.position.z + offset.position.z;
}

type Joint = {
  rotation: Vector3;
  position: Vector3;
};

type JointSnapshot = {
  body: Transform;
  head: Transform;
  rightArm: Transform;
  leftArm: Transform;
  rightLeg: Transform;
  leftLeg: Transform;
};

const snapshots = new WeakMap<object, JointSnapshot>();

function captureTransform(joint: Joint): Transform {
  return {
    rotation: {
      x: joint.rotation.x,
      y: joint.rotation.y,
      z: joint.rotation.z,
    },
    position: {
      x: joint.position.x,
      y: joint.position.y,
      z: joint.position.z,
    },
  };
}

function getSnapshot(
  skin: NonNullable<AnimatedPlayer["skin"]>,
): JointSnapshot {
  const key = skin as object;
  const existing = snapshots.get(key);

  if (existing) return existing;

  const snapshot: JointSnapshot = {
    body: captureTransform(skin.body),
    head: captureTransform(skin.head),
    rightArm: captureTransform(skin.rightArm),
    leftArm: captureTransform(skin.leftArm),
    rightLeg: captureTransform(skin.rightLeg),
    leftLeg: captureTransform(skin.leftLeg),
  };

  snapshots.set(key, snapshot);
  return snapshot;
}

function applyPose(
  skin: NonNullable<AnimatedPlayer["skin"]>,
  pose: Pose,
): void {
  const base = getSnapshot(skin);

  applyTransform(skin.body, base.body, pose.body);
  applyTransform(skin.head, base.head, pose.head);
  applyTransform(skin.rightArm, base.rightArm, pose.rightArm);
  applyTransform(skin.leftArm, base.leftArm, pose.leftArm);
  applyTransform(skin.rightLeg, base.rightLeg, pose.rightLeg);
  applyTransform(skin.leftLeg, base.leftLeg, pose.leftLeg);
}

const NEUTRAL: Pose = {
  body: transform(),
  head: transform(),
  rightArm: transform(),
  leftArm: transform(),
  rightLeg: transform(),
  leftLeg: transform(),
};

/**
 * Brush-shoe gesture.
 *
 * The character bends forward, moves the head forward and down with the
 * torso, reaches toward the shoe with the right arm, lifts the opposing leg,
 * and extends the left arm for balance.
 *
 * Positions are model-space offsets; rotations are in radians.
 */
export const brushShoeGesture: IdleGesture = {
  name: "brush-shoe",
  duration: 5.5,
  weight: 0.45,
  triggers: ["idle"],

  apply(player: AnimatedPlayer, progress: number): void {
    const skin = player.skin;
    if (!skin) return;

    const t = clamp(progress);

    const bent: Pose = {
      body: transform(
        { x: 0.42, z: -0.035 },
        { x: 0.03, y: -0.42, z: 0.2 },
      ),
      head: transform(
        { x: 0.32, y: -0.06, z: 0.025 },
        { x: 0.02, y: -0.12, z: 0.3 },
      ),
      rightArm: transform({ x: -0.18, z: -0.08 }),
      leftArm: transform(
        { x: -0.12, z: 0.42 },
        { x: -0.04, y: 0.02, z: -0.02 },
      ),
      rightLeg: transform({ x: 0.05 }),
      leftLeg: transform({ x: -0.08, z: 0.04 }),
    };

    const reaching: Pose = {
      body: transform(
        { x: 0.5, z: -0.045 },
        { x: 0.04, y: -0.62, z: 0.32 },
      ),
      head: transform(
        { x: 0.58, y: -0.12, z: 0.04 },
        { x: 0.04, y: -0.22, z: 0.52 },
      ),
      rightArm: transform(
        { x: -1.05, y: -0.12, z: -0.22 },
        { x: 0.02, y: -0.08, z: 0.04 },
      ),
      leftArm: transform(
        { x: -0.18, y: 0.04, z: 0.72 },
        { x: -0.06, y: 0.02, z: -0.04 },
      ),
      rightLeg: transform(
        { x: 0.12, z: 0.05 },
        { x: -0.04, y: -0.08, z: -0.02 },
      ),
      leftLeg: transform(
        { x: -0.42, y: 0.08, z: 0.24 },
        { x: -0.16, y: 0.42, z: 0.16 },
      ),
    };

    let pose: Pose;

    if (t < 0.2) {
      pose = interpolatePose(NEUTRAL, bent, t / 0.2);
    } else if (t < 0.35) {
      pose = interpolatePose(bent, reaching, (t - 0.2) / 0.15);
    } else if (t < 0.65) {
      const p = (t - 0.35) / 0.3;
      const brush = Math.sin(p * Math.PI * 6);
      const balance = Math.sin(p * Math.PI * 2);

      pose = {
        ...reaching,
        body: transform(
          {
            x: 0.5 + balance * 0.012,
            z: -0.045,
          },
          {
            x: 0.04,
            y: -0.62 + Math.sin(p * Math.PI) * 0.018,
            z: 0.32,
          },
        ),
        head: transform(
          {
            x: 0.58 + Math.sin(p * Math.PI * 2) * 0.025,
            y: -0.12,
            z: 0.04,
          },
          {
            x: 0.04,
            y: -0.22,
            z: 0.52,
          },
        ),
        rightArm: transform(
          {
            x: -1.05 + brush * 0.13,
            y: -0.12 + brush * 0.035,
            z: -0.22 + brush * 0.055,
          },
          { x: 0.02, y: -0.08, z: 0.04 },
        ),
        leftArm: transform(
          {
            x: -0.18 + balance * 0.035,
            y: 0.04,
            z: 0.72 + balance * 0.04,
          },
          { x: -0.06, y: 0.02, z: -0.04 },
        ),
        rightLeg: reaching.rightLeg,
        leftLeg: transform(
          {
            x: -0.42 + balance * 0.025,
            y: 0.08,
            z: 0.24,
          },
          { x: -0.16, y: 0.42, z: 0.16 },
        ),
      };
    } else {
      pose = interpolatePose(
        reaching,
        NEUTRAL,
        (t - 0.65) / 0.35,
      );
    }

    applyPose(skin, pose);
  },
};
