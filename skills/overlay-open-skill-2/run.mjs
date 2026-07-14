// gated-shell: an overlay skill enforcing Consumed Attenuation over a shell executor.
import { execSync } from 'child_process';

function refuse(reason) {
  console.error(reason);
  process.exit(1);
}

function seal(data) {
  console.log(JSON.stringify(data, null, 2));
  process.exit(0);
}

function parseInput() {
  const inputStr = process.env.RUNX_INPUTS_JSON;
  if (!inputStr) return refuse("No input provided via RUNX_INPUTS_JSON");
  try {
    return JSON.parse(inputStr);
  } catch (e) {
    return refuse("Invalid JSON input");
  }
}

function main() {
  const inputs = parseInput();
  const command = typeof inputs.command === 'string' ? inputs.command.trim() : "";
  let allowedPrefixes = inputs.allowed_prefixes;
  
  if (!command) {
    return refuse("Input 'command' is required.");
  }

  if (typeof allowedPrefixes === 'string') {
    try {
      allowedPrefixes = JSON.parse(allowedPrefixes);
    } catch (e) {
      // ignore, let it fail below
    }
  }

  if (!Array.isArray(allowedPrefixes)) {
    return refuse("Input 'allowed_prefixes' must be a valid JSON array.");
  }

  // GOVERNANCE GATE (Consumed Attenuation)
  let isAllowed = false;
  for (const prefix of allowedPrefixes) {
    if (command.startsWith(prefix)) {
      isAllowed = true;
      break;
    }
  }

  if (!isAllowed) {
    console.error(`Security Policy: Command blocked. Did not match allowed prefixes: ${allowedPrefixes.join(', ')}`);
    process.exit(1);
  }

  // ACTUALLY RUN THE COMMAND (Simulating the upstream runxhq/shell call)
  let output = '';
  try {
    output = execSync(command, { encoding: 'utf-8', stdio: ['pipe', 'pipe', 'pipe'] });
  } catch (e) {
    output = (e.stdout || '') + '\n' + (e.stderr || '');
  }

  seal({
    governance_decision: "ALLOWED",
    executed: true,
    upstream_output: output.trim()
  });
}

main();
