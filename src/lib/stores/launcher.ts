import { writable } from "svelte/store";
import type { KableInstallation } from "../types";

// Installations that have been launched and the one that is currently launching
export const launchedInstallations = writable<KableInstallation[]>([]);
export const currentLaunchingInstallation = writable<KableInstallation | null>(
  null,
);

// Launch state
export const isLaunching = writable(false);
export const launchError = writable<string | null>(null);

// Timeout handle used to auto-clear launching UI after a max wait
export const launchTimeoutHandle = writable<number | null>(null);

// Java state
export const javaStatus = writable<string>("Checking...");
