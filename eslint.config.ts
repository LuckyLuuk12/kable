import tsParser from "@typescript-eslint/parser";
import { TSESLint } from "@typescript-eslint/utils";
import svelte from "eslint-plugin-svelte";
import svelteParser from "svelte-eslint-parser";
import tseslint from "typescript-eslint";

import { apiTryCatchRule } from "./eslint-rules/api-trycatch.js";

function downgradeToWarnings(configs: TSESLint.FlatConfig.Config[]) {
  return configs.map((config): TSESLint.FlatConfig.Config => ({
    ...config,
    rules: Object.fromEntries(
      Object.entries(config.rules ?? {}).map(([name, value]) => [
        name,
        value === "off" || value === 0 ? value : "warn"
      ])
    )
  }));
}

const typescriptRecommended = downgradeToWarnings(
  tseslint.configs.recommended
);

const svelteRecommended = downgradeToWarnings(
  svelte.configs["flat/recommended"]
);

export default [
  {
    ignores: [
      ".svelte-kit/**",
      "node_modules/**",
      "build/**",
      "src-tauri/**",
      "scripts/**",
    ]
  },
  // Svelte rules
  ...svelteRecommended,

  // TypeScript rules only for TypeScript files
  ...typescriptRecommended.map(config => ({
    ...config,
    files: ["**/*.ts"]
  })),

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
      parser: svelteParser,
      parserOptions: {
        parser: tsParser,
      },
    },
    plugins: {
      "@typescript-eslint": tseslint.plugin,
    },
  },
  // Disable TS-specific rules on Svelte files
  {
    files: ["**/*.svelte"],
    rules: {
      "@typescript-eslint/no-unused-vars": "off",
    },
  },

  // Custom rules
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