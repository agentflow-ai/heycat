#!/usr/bin/env bun

import { handleRun } from "./commands/run";
import { handleStatus } from "./commands/status";
import { handleReset } from "./commands/reset";
import { handleHelp } from "./commands/help";
import { handleHookTodoComplete } from "./commands/hook-todo-complete";

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const command = args[0];
  const commandArgs = args.slice(1);

  try {
    switch (command) {
      case "run":
        await handleRun(commandArgs);
        break;
      case "status":
        await handleStatus(commandArgs);
        break;
      case "reset":
        await handleReset(commandArgs);
        break;
      // Hidden hook handler (invoked by Claude Code hook)
      case "hook-todo-complete":
        await handleHookTodoComplete();
        break;
      case "help":
      case "--help":
      case "-h":
      case undefined:
        handleHelp(commandArgs);
        break;
      default:
        console.error(`Unknown command: ${command}`);
        console.error('Run "tcr.ts help" to see available commands');
        process.exit(1);
    }
  } catch (error) {
    if (error instanceof Error) {
      console.error(`Error: ${error.message}`);
    } else {
      console.error("An unexpected error occurred");
    }
    process.exit(1);
  }
}

main();
