export const MODAL_CONTEXT = Symbol();

export interface ModalContext {
  resolve(value?: unknown): void;
  dismiss(): void;
}