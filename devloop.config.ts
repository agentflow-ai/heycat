import { defineConfig } from "devloop/config";

export default defineConfig({
  // TCR (Test-Commit-Refactor) settings
  tcr: {
    maxFailures: 5,        // Max consecutive failures before prompting
    wipPrefix: "WIP: ",    // Prefix for work-in-progress commits
    stateFile: ".tcr-state.json",
  },

  agile: {
    review: {
      // Path to custom review instructions (replaces defaults)
      // The golden path instruction is always appended
      instructionsFile: "",
    },
  },
});
