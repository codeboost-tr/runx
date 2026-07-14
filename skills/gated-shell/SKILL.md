# gated-shell (runx skill)

An overlay skill enforcing a strict "Consumed Attenuation" policy over a generic shell executor.

Instead of acting as a simple duplicate wrapper, this skill **adds a governed capability**: it requires a caller-provided list of allowed command prefixes (`allowed_prefixes`). If the requested `command` does not start with one of the allowed prefixes, the skill blocks execution and exits with a failure, emitting a receipt of the governance decision. 

If allowed, it executes the command (simulating the behavior of the `runxhq/shell` upstream) and returns the output. This allows operators to grant restricted shell access (e.g., only `ls` and `cat`) without exposing arbitrary command execution.

## Inputs
- `command` (string): The shell command to execute.
- `allowed_prefixes` (string): A JSON array of string prefixes.

## Outputs
- `governance_decision` (string): Result of the governance check ("ALLOWED").
- `executed` (boolean): True if the command was executed.
- `upstream_output` (string): The standard output of the executed command.
