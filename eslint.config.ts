import tsParser from "@typescript-eslint/parser";
import svelte from "eslint-plugin-svelte";
import tseslint from "typescript-eslint";
import { apiTryCatchRule } from "./eslint-rules/api-trycatch.js";

export default [
  ...tseslint.configs.recommended,
  ...svelte.configs["flat/recommended"],

  // TypeScript files
  {
    files: ["**/*.ts"],
    languageOptions: {
      parser: tsParser,
    },
  },

  // Svelte files
  {
    files: ["**/*.svelte"],
    languageOptions: {
      parserOptions: {
        parser: tsParser,
      },
    },
  },

  // Your custom rule
  {
    files: ["**/*.{ts,svelte}"],
    plugins: {
      local: {
        rules: {
          "api-try-catch": apiTryCatchRule,
        },
      },
    },
    rules: {
      "local/api-try-catch": "warn",
    },
  },
];