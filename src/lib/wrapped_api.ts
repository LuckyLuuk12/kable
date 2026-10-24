//
// Here we wrap all commands which are generated with Specta into api.ts such that we can use them easier.
// ! Note that TS has no "throws" syntax for functions and IntelliSense won't show warnings that api.<fn> calls should be try-catched.
//

// Import the types (with type prefix) and commands from the generated api.ts file
import { commands } from "./api";
// Import the unwrapAsync function from the utils/result.ts file
import { createApi } from "./utils/result";

// Export the api constant with all commands wrapped in unwrapAsync
export const api = createApi(commands);
