import { defineConfig } from "agile-tcr/config";

export default defineConfig({
  tcr: {
    maxFailures: 5,
    wipPrefix: "WIP: ",
  },
});
