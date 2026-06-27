
export type Result<T, E = unknown> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

/**
 * Extracts the success value or throws on error.
 * Equivalent to Rust `?` after a `match`.
 */
export function unwrap<T, E>(result: Result<T, E>): T {
  if (result.status === "ok") {
    return result.data;
  }

  throw normalizeError(result.error);
}

/**
 * Async version for typedError() results.
 * This is the main function you will use in services.
 *
 * Equivalent to: await result? in Rust-like flow.
 */
export async function unwrapAsync<T, E>(
  promise: Promise<Result<T, E>>
): Promise<T> {
  return unwrap(await promise);
}

/**
 * Returns null instead of throwing.
 * Useful for optional data flows (UI-safe reads).
 */
export function toNullable<T, E>(
  result: Result<T, E>
): T | null {
  return result.status === "ok" ? result.data : null;
}

/**
 * Async nullable version.
 */
export async function toNullableAsync<T, E>(
  promise: Promise<Result<T, E>>
): Promise<T | null> {
  const result = await promise;
  return toNullable(result);
}

/**
 * Checks success without extracting data.
 */
export function isOk<T, E>(
  result: Result<T, E>
): result is Extract<Result<T, E>, { status: "ok" }> {
  return result.status === "ok";
}

/**
 * Checks error state.
 */
export function isError<T, E>(
  result: Result<T, E>
): result is Extract<Result<T, E>, { status: "error" }> {
  return result.status === "error";
}

/**
 * Normalizes unknown backend errors into a real Error object.
 * Prevents throwing raw strings / objects from IPC layer.
 */
function normalizeError(error: unknown): Error {
  if (error instanceof Error) return error;

  if (typeof error === "string") {
    return new Error(error);
  }

  try {
    return new Error(JSON.stringify(error));
  } catch {
    return new Error("Unknown error");
  }
}
