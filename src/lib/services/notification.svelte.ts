import type { Service } from "./app.svelte";

export type NotificationType = "success" | "error" | "warning" | "info";

export interface Notification {
  id: string;
  message: string;
  type: NotificationType;
  duration: number; // milliseconds
  timestamp: Date;
  isHovered: boolean;
  markdown?: boolean; // if true, parse message as markdown/html
  onClick?: () => void | Promise<void>; // optional click handler
}

export class NotificationService implements Service {
  DEFAULT_DURATION = $state(5000); // 5 seconds
  MAX_HISTORY = $state(100); // Keep last 100 notifications
  public notificationHistory = $state<Notification[]>([]);
  public notifications = $state<Notification[]>([]);

  init(): Promise<void> | void {}
  restart?(): Promise<void> | void {
    this.notifications = [];
    this.notificationHistory = [];
  }
  destroy(): Promise<void> | void {
    this.notifications = [];
    this.notificationHistory = [];
  }
  /**
   * Send a notification
   * @param message The notification message (can be markdown/html if markdown=true)
   * @param type The notification type
   * @param duration Duration in seconds (or milliseconds if > 100). Default: 5 seconds
   * @param markdown Whether to parse message as markdown/html. Default: false
   * @param onClick Optional click handler for the notification
   * @returns The notification ID
   */
  send(message: string, type: NotificationType = "info", duration?: number, markdown: boolean = false, onClick?: () => void | Promise<void>): string {
    const id = crypto.randomUUID();

    // Convert duration: if <= 100, treat as seconds; otherwise milliseconds
    let durationMs = this.DEFAULT_DURATION;
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
      onClick,
    };

    // Add to active notifications
    this.notifications = [...this.notifications, notification];

    // Add to history
    this.notificationHistory = [notification, ...this.notificationHistory].slice(0, this.MAX_HISTORY);

    // Auto-dismiss after duration (unless hovered)
    this.scheduleDismiss(id, durationMs);

    return id;
  }

  /**
   * Shorthand methods for common notification types
   */
  success(message: string, duration?: number, markdown: boolean = false, onClick?: () => void | Promise<void>): string {
    return this.send(message, "success", duration, markdown, onClick);
  }

  error(message: string, duration?: number, markdown: boolean = false, onClick?: () => void | Promise<void>): string {
    return this.send(message, "error", duration, markdown, onClick);
  }

  warning(message: string, duration?: number, markdown: boolean = false, onClick?: () => void | Promise<void>): string {
    return this.send(message, "warning", duration, markdown, onClick);
  }

  info(message: string, duration?: number, markdown: boolean = false, onClick?: () => void | Promise<void>): string {
    return this.send(message, "info", duration, markdown, onClick);
  }

  /**
   * Dismiss a specific notification
   */
  dismiss(id: string): void {
    this.notifications = this.notifications.filter((notification) => notification.id !== id);
  }

  /**
   * Dismiss all active notifications
   */
  dismissAll(): void {
    this.notifications = [];
  }

  /**
   * Clear notification history
   */
  clearHistory(): void {
    this.notificationHistory = [];
  }

  /**
   * Set hover state for a notification (prevents auto-dismiss)
   */
  setHovered(id: string, isHovered: boolean): void {
    this.notifications = this.notifications.map((notification) => (notification.id === id ? { ...notification, isHovered } : notification));
  }

  /**
   * Schedule auto-dismiss for a notification
   */
  private scheduleDismiss(id: string, duration: number): void {
    setTimeout(() => {
      const currentNotifications = this.notifications;
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
