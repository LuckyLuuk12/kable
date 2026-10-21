import * as skinview3d from "skinview3d";

import { log } from "$lib";
import { brushShoeGesture } from "./brushShoe";
import { danceGesture } from "./dance";
import { lookAroundGesture } from "./lookAround";
import { pointGesture } from "./point";
import { stretchGesture } from "./stretch";

export const DEFAULT_GESTURES = [
  lookAroundGesture,
  pointGesture,
  stretchGesture,
  brushShoeGesture,
  danceGesture,
];

/**
 * The player type is inferred from skinview3d's FunctionAnimation API.
 */
export type AnimatedPlayer = Parameters<
  InstanceType<typeof skinview3d.FunctionAnimation>["fn"]
>[0];

export type AnimationTrigger =
  | "idle"
  | "hover"
  | "click"
  | "drag-start"
  | "drag"
  | "drag-end"
  | "keybind";

export interface IdleGesture {
  name: string;
  duration: number;
  weight: number;

  /** Events that are allowed to trigger this gesture. */
  triggers: AnimationTrigger[];

  /** Optional keyboard shortcut, for easter eggs. */
  keybind?: string;

  apply(player: AnimatedPlayer, progress: number): void;
}

export type ViewerInteraction =
  | "hover"
  | "click"
  | "drag-start"
  | "drag"
  | "drag-end"
  | "leave";

export interface InteractionGesture {
  /** Interaction that triggers this reaction. */
  interaction: ViewerInteraction;

  /** Name of the gesture to play. */
  gesture: string;

  /** Higher-priority requests are processed first. */
  priority?: number;

  /** Minimum time between accepted requests of this interaction, in seconds. */
  cooldown?: number;
}

export interface IdleOrchestratorOptions {
  /** Initial delay before the first ambient gesture, in seconds. */
  initialDelayMin?: number;
  initialDelayMax?: number;

  /** Random pause between gestures, in seconds. */
  cooldownMin?: number;
  cooldownMax?: number;

  /** Whether ambient gesture scheduling is initially enabled. */
  enabled?: boolean;

  /** Whether to schedule ambient gestures while the pointer is hovering. */
  ambientWhileHovered?: boolean;

  /** Whether to schedule ambient gestures during pointer manipulation. */
  ambientDuringDrag?: boolean;

  /** Minimum time between accepted interaction requests, in seconds. */
  interactionCooldown?: number;
}

const DEFAULT_OPTIONS: Required<IdleOrchestratorOptions> = {
  initialDelayMin: 4,
  initialDelayMax: 10,
  cooldownMin: 8,
  cooldownMax: 20,
  enabled: true,
  ambientWhileHovered: false,
  ambientDuringDrag: false,
  interactionCooldown: 0.25,
};

/**
 * Default interaction-to-gesture mappings.
 *
 * Mappings only trigger gestures that explicitly allow the corresponding
 * AnimationTrigger. A missing gesture or incompatible trigger is ignored.
 */
export const DEFAULT_INTERACTION_GESTURES: InteractionGesture[] = [
  { interaction: "hover", gesture: "stretch", priority: 1, cooldown: 2 },
  { interaction: "click", gesture: "look-around", priority: 2, cooldown: 0.5 },
  { interaction: "drag-start", gesture: "look-around", priority: 2, cooldown: 1 },
  { interaction: "drag", gesture: "point", priority: 3, cooldown: 0.5 },
];

interface PendingGesture {
  gesture: IdleGesture;
  priority: number;
  interaction?: ViewerInteraction;
}

export class IdleOrchestrator {
  private readonly animation: InstanceType<typeof skinview3d.IdleAnimation>;
  private readonly gestures: IdleGesture[];
  private readonly interactions: InteractionGesture[];
  private readonly options: Required<IdleOrchestratorOptions>;

  private animationId: number | undefined;

  private activeGesture: IdleGesture | null = null;
  private activeGestureStartedAt = 0;
  private activeGesturePriority = 0;

  private readonly pendingGestures: PendingGesture[] = [];
  private lastInteractionAt = new Map<ViewerInteraction, number>();

  /**
   * Elapsed time reported by this orchestrator's registered animation
   * callback. All gesture timestamps use this same timebase.
   */
  private currentTime = 0;

  /** Earliest time at which another ambient gesture may start. */
  private nextGestureAt = 0;

  private enabled: boolean;
  private hovered = false;
  private dragging = false;
  private disposed = false;

  constructor(
    animation: InstanceType<typeof skinview3d.IdleAnimation>,
    gestures: IdleGesture[],
    options: IdleOrchestratorOptions = {},
    interactions: InteractionGesture[] = DEFAULT_INTERACTION_GESTURES,
  ) {
    this.animation = animation;
    this.gestures = gestures.filter(
      (gesture) =>
        Number.isFinite(gesture.duration) &&
        gesture.duration > 0 &&
        Number.isFinite(gesture.weight) &&
        gesture.weight > 0,
    );
    this.interactions = [...interactions];

    this.options = {
      ...DEFAULT_OPTIONS,
      ...options,
    };

    this.validateOptions();

    this.enabled = this.options.enabled;
    this.nextGestureAt = this.randomBetween(
      this.options.initialDelayMin,
      this.options.initialDelayMax,
    );

    this.animationId = this.animation.addAnimation((player, elapsed) => {
      this.update(player, elapsed);
    });
  }

  private validateOptions(): void {
    const {
      initialDelayMin,
      initialDelayMax,
      cooldownMin,
      cooldownMax,
      interactionCooldown,
    } = this.options;

    if (
      initialDelayMin < 0 ||
      initialDelayMax < initialDelayMin ||
      cooldownMin < 0 ||
      cooldownMax < cooldownMin ||
      interactionCooldown < 0
    ) {
      throw new RangeError(
        "Invalid orchestrator timing options: delays and cooldowns must be non-negative, with maximums >= minimums.",
      );
    }
  }

  /**
   * Advance the current gesture and process the scheduler.
   *
   * Only this method advances an active gesture. Interaction handlers
   * enqueue requests instead of starting or replacing animations directly.
   */
  private update(player: AnimatedPlayer, elapsed: number): void {
    if (this.disposed) return;

    this.currentTime = elapsed;

    if (this.activeGesture) {
      const gesture = this.activeGesture;
      const gestureElapsed = Math.max(
        0,
        this.currentTime - this.activeGestureStartedAt,
      );
      const progress = Math.min(
        1,
        gestureElapsed / gesture.duration,
      );

      gesture.apply(player, progress);

      if (progress < 1) {
        return;
      }

      // The gesture has completed. Do not start another gesture from
      // inside its completion branch; scheduling below owns that decision.
      this.activeGesture = null;
      this.activeGesturePriority = 0;

      // The ambient cooldown starts after every completed gesture.
      this.scheduleNextGesture(this.currentTime);
    }

    // Explicit interactions take precedence over ambient scheduling.
    const pending = this.takeNextPendingGesture();

    if (pending) {
      this.startGesture(pending.gesture, this.currentTime, pending.priority);
      return;
    }

    if (!this.canRunAmbientGestures()) return;
    if (this.currentTime < this.nextGestureAt) return;

    const gesture = this.chooseGesture();

    if (!gesture) {
      this.scheduleNextGesture(this.currentTime);
      return;
    }

    this.startGesture(gesture, this.currentTime, 0);
  }

  private canRunAmbientGestures(): boolean {
    if (!this.enabled) return false;
    if (this.dragging && !this.options.ambientDuringDrag) return false;
    if (this.hovered && !this.options.ambientWhileHovered) return false;

    return this.gestures.some((gesture) =>
      gesture.triggers.includes("idle"),
    );
  }

  /**
   * Start a gesture. This is only called by the centralized scheduler.
   */
  private startGesture(
    gesture: IdleGesture,
    time: number,
    priority: number,
  ): boolean {
    if (this.disposed || this.activeGesture) return false;

    this.activeGesture = gesture;
    this.activeGestureStartedAt = time;
    this.activeGesturePriority = priority;

    return true;
  }

  /**
   * Enqueue a mapped interaction reaction.
   *
   * A request is not allowed to interrupt the active gesture. Repeated
   * requests for the same interaction are coalesced in the queue.
   */
  trigger(interaction: ViewerInteraction): boolean {
    if (this.disposed) return false;

    this.updateInteractionState(interaction);

    // Leaving the viewer only updates pointer state.
    if (interaction === "leave") return false;

    const mapping = this.interactions
      .filter((item) => item.interaction === interaction)
      .sort((a, b) => (b.priority ?? 0) - (a.priority ?? 0))
      .find((item) => {
        const gesture = this.gestures.find(
          (candidate) => candidate.name === item.gesture,
        );

        const requiredTrigger = this.triggerForInteraction(interaction);

        return (
          gesture !== undefined &&
          requiredTrigger !== undefined &&
          gesture.triggers.includes(requiredTrigger)
        );
      });

    if (!mapping) return false;

    const gesture = this.gestures.find(
      (item) => item.name === mapping.gesture,
    );

    if (!gesture) return false;

    const lastTime = this.lastInteractionAt.get(interaction);
    const cooldown = Math.max(
      this.options.interactionCooldown,
      mapping.cooldown ?? 0,
    );

    if (
      lastTime !== undefined &&
      this.currentTime - lastTime < cooldown
    ) {
      return false;
    }

    // Do not flood the queue with repeated hover/drag events.
    if (
      this.pendingGestures.some(
        (item) => item.interaction === interaction,
      )
    ) {
      return false;
    }

    this.pendingGestures.push({
      gesture,
      priority: mapping.priority ?? 0,
      interaction,
    });

    this.lastInteractionAt.set(interaction, this.currentTime);
    log(`Queued gesture "${gesture.name}" for interaction "${interaction}"`);
    return true;
  }

  private triggerForInteraction(
    interaction: ViewerInteraction,
  ): AnimationTrigger | undefined {
    switch (interaction) {
      case "hover":
        return "hover";
      case "click":
        return "click";
      case "drag-start":
        return "drag-start";
      case "drag":
        return "drag";
      case "drag-end":
        return "drag-end";
      case "leave":
        return undefined;
    }
  }

  private updateInteractionState(interaction: ViewerInteraction): void {
    switch (interaction) {
      case "hover":
        this.hovered = true;
        break;

      case "leave":
        this.hovered = false;
        this.dragging = false;
        break;

      case "drag-start":
      case "drag":
        this.dragging = true;
        break;

      case "drag-end":
        this.dragging = false;
        break;

      case "click":
        break;
    }
  }

  /**
   * Remove the highest-priority pending request.
   * Stable ordering preserves FIFO order for equal priorities.
   */
  private takeNextPendingGesture(): PendingGesture | undefined {
    if (this.pendingGestures.length === 0) return undefined;

    let bestIndex = 0;

    for (let i = 1; i < this.pendingGestures.length; i++) {
      if (
        this.pendingGestures[i].priority >
        this.pendingGestures[bestIndex].priority
      ) {
        bestIndex = i;
      }
    }

    return this.pendingGestures.splice(bestIndex, 1)[0];
  }

  /**
   * Select an ambient gesture using weighted random selection.
   */
  private chooseGesture(): IdleGesture | null {
    const eligible = this.gestures.filter((gesture) =>
      gesture.triggers.includes("idle"),
    );

    const totalWeight = eligible.reduce(
      (sum, gesture) => sum + gesture.weight,
      0,
    );

    if (totalWeight <= 0) return null;

    let choice = Math.random() * totalWeight;

    for (const gesture of eligible) {
      choice -= gesture.weight;

      if (choice < 0) return gesture;
    }

    return eligible[eligible.length - 1] ?? null;
  }

  private scheduleNextGesture(time: number): void {
    this.nextGestureAt =
      time +
      this.randomBetween(
        this.options.cooldownMin,
        this.options.cooldownMax,
      );
  }

  private randomBetween(min: number, max: number): number {
    return min + Math.random() * (max - min);
  }

  /**
   * Enable or disable ambient scheduling.
   * Explicit interactions remain available when ambient scheduling is off.
   */
  setEnabled(enabled: boolean): void {
    if (this.disposed) return;

    this.enabled = enabled;

    if (enabled) {
      this.nextGestureAt = this.currentTime + this.randomBetween(
        this.options.initialDelayMin,
        this.options.initialDelayMax,
      );
    } else {
      // Remove queued ambient work; interaction requests remain queued.
      // The active gesture is allowed to finish normally.
      this.pendingGestures.splice(
        0,
        this.pendingGestures.length,
        ...this.pendingGestures.filter((item) => item.interaction !== undefined),
      );
    }
  }

  /**
   * Convenience methods for viewer pointer events.
   */
  onHover(): boolean {
    return this.trigger("hover");
  }

  onClick(): boolean {
    return this.trigger("click");
  }

  onDragStart(): boolean {
    return this.trigger("drag-start");
  }

  onDrag(): boolean {
    return this.trigger("drag");
  }

  onDragEnd(): boolean {
    return this.trigger("drag-end");
  }

  onLeave(): boolean {
    return this.trigger("leave");
  }

  /**
   * Queue a gesture registered for the specified keyboard shortcut.
   */
  onKeybind(key: string): boolean {
    if (this.disposed) return false;

    const normalizedKey = key.toLowerCase();

    const gesture = this.gestures.find(
      (item) =>
        item.triggers.includes("keybind") &&
        item.keybind?.toLowerCase() === normalizedKey,
    );

    if (!gesture) return false;

    this.pendingGestures.push({
      gesture,
      priority: 2,
    });

    return true;
  }

  /**
   * Queue a named gesture for development/testing.
   * Manual playback bypasses trigger restrictions, but not serialization.
   */
  play(name: string): boolean {
    if (this.disposed) return false;

    const gesture = this.gestures.find((item) => item.name === name);

    if (!gesture) return false;

    this.pendingGestures.push({
      gesture,
      priority: Number.MAX_SAFE_INTEGER,
    });

    return true;
  }

  /**
   * Cancel the active gesture and clear all pending work.
   */
  reset(): void {
    this.activeGesture = null;
    this.activeGesturePriority = 0;
    this.pendingGestures.length = 0;
    this.hovered = false;
    this.dragging = false;

    this.scheduleNextGesture(this.currentTime);
  }

  /**
   * Remove this orchestrator's callback from the animation.
   */
  dispose(): void {
    if (this.disposed) return;

    this.disposed = true;
    this.activeGesture = null;
    this.activeGesturePriority = 0;
    this.pendingGestures.length = 0;

    this.animation.removeAnimation(this.animationId);
    this.animationId = undefined;
    this.lastInteractionAt.clear();
  }
}