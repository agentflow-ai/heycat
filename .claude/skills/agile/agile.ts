#!/usr/bin/env bun

import { handleCreate } from "./commands/create";
import { handleMove } from "./commands/move";
import { handleList } from "./commands/list";
import { handleArchive } from "./commands/archive";
import { handleDelete } from "./commands/delete";
import { handleWork } from "./commands/work";
import { handleSpec } from "./commands/spec";
import { handleGuidance } from "./commands/guidance";
import { handleHelp } from "./commands/help";

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const command = args[0];
  const commandArgs = args.slice(1);

  try {
    switch (command) {
      case "create":
        await handleCreate(commandArgs);
        break;
      case "move":
        await handleMove(commandArgs);
        break;
      case "list":
        await handleList(commandArgs);
        break;
      case "archive":
        await handleArchive(commandArgs);
        break;
      case "delete":
        await handleDelete(commandArgs);
        break;
      case "work":
        await handleWork(commandArgs);
        break;
      case "spec":
        await handleSpec(commandArgs);
        break;
      case "guidance":
        await handleGuidance(commandArgs);
        break;
      case "help":
      case "--help":
      case "-h":
      case undefined:
        handleHelp(commandArgs);
        break;
      default:
        console.error(`Unknown command: ${command}`);
        console.error('Run "agile.ts help" to see available commands');
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
