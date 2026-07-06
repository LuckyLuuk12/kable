import tsParser from "@typescript-eslint/parser";
import { TSESLint } from "@typescript-eslint/utils";
import svelte from "eslint-plugin-svelte";
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

const recommended = downgradeToWarnings(tseslint.configs.recommended);
const svelteRecommended = downgradeToWarnings(
  svelte.configs["flat/recommended"]
);

export default [
  ...recommended,
  ...svelteRecommended,

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