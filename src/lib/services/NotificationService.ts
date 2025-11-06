import { writable, get } from "svelte/store";

export type NotificationType = "success" | "error" | "warning" | "info";

export interface Notification {
  id: string;
  message: string;
  type: NotificationType;
  duration: number; // milliseconds
  timestamp: Date;
  isHovered: boolean;
  markdown?: boolean; // if true, parse message as markdown/html
}

// Store for active notifications
export const notifications = writable<Notification[]>([]);

// Store for notification history (for tray)
export const notificationHistory = writable<Notification[]>([]);

const DEFAULT_DURATION = 5000; // 5 seconds
const MAX_HISTORY = 100; // Keep last 100 notifications

export class NotificationService {
  /**
   * Send a notification
   * @param message The notification message (can be markdown/html if markdown=true)
   * @param type The notification type
   * @param duration Duration in seconds (or milliseconds if > 100). Default: 5 seconds
   * @param markdown Whether to parse message as markdown/html. Default: false
   * @returns The notification ID
   */
  static send(
    message: string,
    type: NotificationType = "info",
    duration?: number,
    markdown: boolean = false,
  ): string {
    const id = crypto.randomUUID();
    
    // Convert duration: if <= 100, treat as seconds; otherwise milliseconds
    let durationMs = DEFAULT_DURATION;
    if (duration !== undefined) {
      durationMs = duration <= 100 ? duration * 1000 : duration;
    }

    const notification: Notification = {
      id,
      message,
      type,
      duration: durationMs,
      timestamp: new Date(),
      isHovered: false,
      markdown,
    };

    // Add to active notifications
    notifications.update((n) => [...n, notification]);

    // Add to history
    notificationHistory.update((history) => {
      const newHistory = [notification, ...history];
      // Keep only last MAX_HISTORY items
      return newHistory.slice(0, MAX_HISTORY);
    });

    // Auto-dismiss after duration (unless hovered)
    this.scheduleDismiss(id, durationMs);

    return id;
  }

  /**
   * Shorthand methods for common notification types
   */
  static success(message: string, duration?: number, markdown: boolean = false): string {
    return this.send(message, "success", duration, markdown);
  }

  static error(message: string, duration?: number, markdown: boolean = false): string {
    return this.send(message, "error", duration, markdown);
  }

  static warning(message: string, duration?: number, markdown: boolean = false): string {
    return this.send(message, "warning", duration, markdown);
  }

  static info(message: string, duration?: number, markdown: boolean = false): string {
    return this.send(message, "info", duration, markdown);
  }

  /**
   * Dismiss a specific notification
   */
  static dismiss(id: string): void {
    notifications.update((n) => n.filter((notification) => notification.id !== id));
  }

  /**
   * Dismiss all active notifications
   */
  static dismissAll(): void {
    notifications.set([]);
  }

  /**
   * Clear notification history
   */
  static clearHistory(): void {
    notificationHistory.set([]);
  }

  /**
   * Set hover state for a notification (prevents auto-dismiss)
   */
  static setHovered(id: string, isHovered: boolean): void {
    notifications.update((n) =>
      n.map((notification) =>
        notification.id === id ? { ...notification, isHovered } : notification,
      ),
    );
  }

  /**
   * Schedule auto-dismiss for a notification
   */
  private static scheduleDismiss(id: string, duration: number): void {
    setTimeout(() => {
      const currentNotifications = get(notifications);
      const notification = currentNotifications.find((n) => n.id === id);
      
      // Only dismiss if not hovered
      if (notification && !notification.isHovered) {
        this.dismiss(id);
      } else if (notification && notification.isHovered) {
        // If hovered, reschedule dismiss check
        this.scheduleDismiss(id, 1000); // Check again in 1 second
      }
    }, duration);
  }
}
