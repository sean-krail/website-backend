module.exports = {
  root: true,
  parser: "@typescript-eslint/parser",
  parserOptions: {
    project: "./tsconfig.json",
  },
  plugins: ["@typescript-eslint"],
  extends: ["airbnb-typescript/base", "prettier", "plugin:import/recommended"],
  rules: {
    "import/no-unresolved": "warn",
  },
};
