import type { Component } from "svelte";

export const MODAL_CONTEXT = Symbol();

export interface ModalContext {
  resolve(value?: unknown): void;
  dismiss(): void;
}

export type ModalInstance = {
  id: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  component: Component<any>;
  props: Record<string, unknown>;
  resolve: (value: unknown) => void;
};