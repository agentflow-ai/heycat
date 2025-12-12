import { defineConfig } from "devloop/config";

export default defineConfig({
  tcr: {
    maxFailures: 5,
    wipPrefix: "WIP: ",
  },
});
