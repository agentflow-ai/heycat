import { resetFailures, loadState } from "../lib/state";
import { findProjectRoot, clearErrorLog, readErrorLog } from "../lib/utils";

export async function handleReset(): Promise<void> {
  const projectRoot = await findProjectRoot();
  const state = await loadState(projectRoot);
  const errors = await readErrorLog(projectRoot);

  const previousCount = state.failureCount;
  const hadErrors = errors.length > 0;

  await resetFailures(projectRoot);
  await clearErrorLog(projectRoot);

  if (previousCount > 0 || hadErrors) {
    if (previousCount > 0) {
      console.log(`TCR: Failure counter reset (was ${previousCount})`);
    }
    if (hadErrors) {
      console.log(`TCR: Error log cleared (had ${errors.length} entries)`);
    }
    console.log("You can continue working on the current task.");
  } else {
    console.log("TCR: Failure counter was already at 0, no errors to clear");
  }
}
