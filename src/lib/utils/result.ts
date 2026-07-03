// export type Result<T, E = unknown> =
//   | { status: "ok"; data: T }
//   | { status: "error"; error: E };

// /**
//  * Extracts the success value or throws on error.
//  * Equivalent to Rust `?` after a `match`.
//  */
// export function unwrap<T, E>(result: Result<T, E>): T {
//   if (result.status === "ok") return result.data;

//   const err = normalizeError(result.error);
//   (err as any).original = result.error;
//   throw err;
// }

// /**
//  * Async version for typedError() results.
//  * This is the main function you will use in services.
//  *
//  * Equivalent to: await result? in Rust-like flow.
//  */
// export async function unwrapAsync<T, E>(
//   promise: Promise<Result<T, E>>
// ): Promise<T> {
//   return unwrap(await promise);
// }

// /**
//  * Returns null instead of throwing.
//  * Useful for optional data flows (UI-safe reads).
//  */
// export function toNullable<T, E>(
//   result: Result<T, E>
// ): T | null {
//   return result.status === "ok" ? result.data : null;
// }

// /**
//  * Async nullable version.
//  */
// export async function toNullableAsync<T, E>(
//   promise: Promise<Result<T, E>>
// ): Promise<T | null> {
//   const result = await promise;
//   return toNullable(result);
// }

// /**
//  * Checks success without extracting data.
//  */
// export function isOk<T, E>(
//   result: Result<T, E>
// ): result is Extract<Result<T, E>, { status: "ok" }> {
//   return result.status === "ok";
// }

// /**
//  * Checks error state.
//  */
// export function isError<T, E>(
//   result: Result<T, E>
// ): result is Extract<Result<T, E>, { status: "error" }> {
//   return result.status === "error";
// }

// /**
//  * Normalizes unknown backend errors into a real Error object.
//  * Prevents throwing raw strings / objects from IPC layer.
//  */
// function normalizeError(error: unknown): Error {
//   if (error instanceof Error) return error;

//   if (typeof error === "string") {
//     return new Error(error);
//   }

//   try {
//     return new Error(JSON.stringify(error));
//   } catch {
//     return new Error("Unknown error");
//   }
// }

// export function map<T, U, E>(
//   result: Result<T, E>,
//   fn: (t: T) => U
// ): Result<U, E> {
//   return result.status === "ok"
//     ? { status: "ok", data: fn(result.data) }
//     : result;
// }
// export async function mapAsync<T, U, E>(
//   promise: Promise<Result<T, E>>,
//   fn: (t: T) => U | Promise<U>
// ): Promise<Result<U, E>> {
//   const r = await promise;

//   if (r.status === "error") return r;

//   return { status: "ok", data: await fn(r.data) };
// }
// type UnwrapResultPromise<T> =
//   T extends Promise<Result<infer R, any>>
//   ? R
//   : never;
// type ApiFn<F> =
//   F extends (...args: infer A) => Promise<Result<infer R, any>>
//   ? (...args: A) => Promise<R>
//   : never;
// type ApiFromCommands<T> = {
//   [K in keyof T]:
//   T[K] extends (...args: any[]) => any
//   ? ApiFn<T[K]>
//   : never;
// };
// export function createApi<T extends Record<string, any>>(
//   commands: T
// ): ApiFromCommands<T> {
//   const api: any = {};

//   for (const key in commands) {
//     const fn = commands[key];

//     api[key] = async (...args: any[]) => {
//       const result = await fn(...args);

//       if (result.status === "error") {
//         throw normalizeError(result.error);
//       }

//       return result.data;
//     };
//   }

//   return api;
// }

// ? OLD ^^

/* =========================
   Result type
========================= */

export type Result<T, E = unknown> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

/* =========================
   Error normalization
========================= */

function normalizeError(error: unknown): Error {
  if (error instanceof Error) return error;

  if (typeof error === "string") return new Error(error);

  try {
    return new Error(JSON.stringify(error));
  } catch {
    return new Error("Unknown error");
  }
}

/* =========================
   Core unwrap utilities
========================= */

export function unwrap<T, E>(result: Result<T, E>): T {
  if (result.status === "ok") return result.data;

  const err = normalizeError(result.error);
  (err as any).original = result.error;
  throw err;
}

export async function unwrapAsync<T, E>(
  promise: Promise<Result<T, E>>,
): Promise<T> {
  return unwrap(await promise);
}

export function toNullable<T, E>(result: Result<T, E>): T | null {
  return result.status === "ok" ? result.data : null;
}

export async function toNullableAsync<T, E>(
  promise: Promise<Result<T, E>>,
): Promise<T | null> {
  return toNullable(await promise);
}

/* =========================
   Type guards
========================= */

export function isOk<T, E>(
  result: Result<T, E>,
): result is Extract<Result<T, E>, { status: "ok" }> {
  return result.status === "ok";
}

export function isError<T, E>(
  result: Result<T, E>,
): result is Extract<Result<T, E>, { status: "error" }> {
  return result.status === "error";
}

/* =========================
   Result mapping helpers
========================= */

export function map<T, U, E>(
  result: Result<T, E>,
  fn: (t: T) => U,
): Result<U, E> {
  return result.status === "ok"
    ? { status: "ok", data: fn(result.data) }
    : result;
}

export async function mapAsync<T, U, E>(
  promise: Promise<Result<T, E>>,
  fn: (t: T) => U | Promise<U>,
): Promise<Result<U, E>> {
  const r = await promise;

  if (r.status === "error") return r;

  return {
    status: "ok",
    data: await fn(r.data),
  };
}

/* =========================
   API transformation types
   (commands -> api)
========================= */

type ApiFn<F> = F extends (...args: infer A) => Promise<Result<infer R, any>>
  ? (...args: A) => Promise<R>
  : never;

export type ApiFromCommands<T> = {
  [K in keyof T]: T[K] extends (...args: any[]) => any ? ApiFn<T[K]> : never;
};

/* =========================
   createApi runtime wrapper
========================= */

export function createApi<
  T extends Record<string, (...args: any[]) => Promise<Result<any, any>>>,
>(commands: T): ApiFromCommands<T> {
  const api: Partial<ApiFromCommands<T>> = {};

  for (const key in commands) {
    const fn = commands[key];

    api[key] = (async (...args: any[]) => {
      const res = await fn(...args);

      if (res.status === "error") {
        throw normalizeError(res.error);
      }

      return res.data;
    }) as ApiFromCommands<T>[typeof key];
  }

  return api as ApiFromCommands<T>;
}
