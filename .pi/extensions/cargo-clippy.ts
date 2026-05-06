import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { isToolCallEventType } from "@mariozechner/pi-coding-agent";

export default function (pi: ExtensionAPI) {
  // Track edited files per session (all turns) - use Set to deduplicate
  const editedRustFiles = new Set<string>();
  // Track if clippy has already run for current changes
  let clippyRan = false;

  // Track file edits throughout the session
  pi.on("tool_call", async (event, ctx) => {
    // Check for edit tool
    if (isToolCallEventType("edit", event)) {
      const path = event.input.path;
      if (path.endsWith(".rs")) {
        editedRustFiles.add(path);
        // ctx.ui.notify(`Tracked rust file: ${path}`, "info");
        // New edits mean we need to run clippy again
        clippyRan = false;
      }
    }

    // Check for write tool
    if (isToolCallEventType("write", event)) {
      const path = event.input.path;
      if (path.endsWith(".rs")) {
        editedRustFiles.add(path);
        // ctx.ui.notify(`Tracked rust file: ${path}`, "info");
        // New edits mean we need to run clippy again
        clippyRan = false;
      }
    }
  });

  // Run check and clippy only at the end of the agent processing (agent_end)
  // This fires once per user prompt, after all turns are complete
  pi.on("agent_end", async (event, ctx) => {
    // Skip if no Rust files were edited
    if (editedRustFiles.size === 0) {
      return;
    }

    // Skip if clippy already ran for current changes
    if (clippyRan) {
      return;
    }

    // Mark that we've run clippy - will reset if new edits happen in next prompt
    clippyRan = true;

    const uniqueFiles = [...editedRustFiles];
    ctx.ui.notify(`🔍 Running cargo check on ${uniqueFiles.length} edited file(s): ${uniqueFiles.join(", ")}`, "info");

    // First, run cargo check to catch compilation errors
    ctx.ui.notify(`🔨 Running cargo check...`, "info");
    
    const checkResult = await pi.exec("cargo", ["check", "-q"], {
      cwd: ctx.cwd,
      timeout: 120000, // 2 minutes
    });

    // If there are compilation errors, force agent to fix them
    if (checkResult.code !== 0) {
      const output = checkResult.stderr || checkResult.stdout;
      
      ctx.ui.notify(`❌ Compilation errors found!`, "error");
      ctx.ui.notify(output, "error");
      
      // Pass raw output to the LLM asking to fix
      await pi.sendUserMessage([
        {
          type: "text",
          text: `Fix the following Rust compilation errors:\n\n${output}\n\nRun \`cargo check\` to verify after fixing.`
        }
      ], { deliverAs: "followUp" });
      
      // Keep tracking - agent needs to fix
      clippyRan = false;
      return;
    }

    ctx.ui.notify(`✅ Cargo check passed!`, "success");

    ctx.ui.notify(`🔧 Running cargo clippy...`, "info");

    // Try to auto-fix issues with cargo clippy --fix
    const fixResult = await pi.exec("cargo", [
      "clippy",
      "--fix",
      "--allow-dirty",
      "--allow-staged",
    ], {
      cwd: ctx.cwd,
      timeout: 180000, // 3 minutes
    });

    // Check if any fixes were applied
    if (fixResult.stdout.includes("Fixed") || fixResult.stderr.includes("Fixed")) {
      ctx.ui.notify("✅ Applied clippy fixes automatically", "success");
    }

    // Now run clippy to check for issues
    const clippyResult = await pi.exec("cargo", [
      "clippy",
      "-q", // no compilation messages
      "--all-features"
    ], {
      cwd: ctx.cwd,
      timeout: 120000, // 2 minutes
    });

    // If clippy found issues, force agent to fix them
    if (clippyResult.code !== 0) {
      const output = clippyResult.stderr || clippyResult.stdout;
      
      ctx.ui.notify(`⚠️ Clippy found issues!`, "warning");
      ctx.ui.notify(output, "warning");
      
      // Pass raw output to the LLM asking to fix
      await pi.sendUserMessage([
        {
          type: "text",
          text: `Fix the following clippy warnings/errors:\n\n${output}\n\nRun \`cargo clippy --all-features\` to verify after fixing.`
        }
      ], { deliverAs: "followUp" });
      
      // Keep tracking - agent needs to fix
      clippyRan = false;
    } else {
      ctx.ui.notify(`✅ Cargo clippy passed!`, "success");
      
      // Clear tracking now that we're done
      editedRustFiles.clear();
    }
  });
}