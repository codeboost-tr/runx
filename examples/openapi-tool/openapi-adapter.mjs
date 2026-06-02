// OpenAPI external-adapter subprocess (runx.external_adapter.v1).
//
// Turns one OpenAPI operation into a governed tool call: it reads the invocation
// frame from stdin, loads the checked-in OpenAPI spec, resolves the requested
// operationId, validates the required parameters, and emits the resolved HTTP
// request (method + URL) as the sealed result. The network leg lives on the
// adapter (the supervised side of the boundary); this example resolves the
// governed request deterministically rather than calling out, so it dogfoods
// with no network and no key. A production adapter performs the call here.
import { readFileSync } from "node:fs";

let input = "";
process.stdin.on("data", (chunk) => {
  input += chunk;
});
process.stdin.on("end", () => {
  let invocation = {};
  try {
    invocation = JSON.parse(input.trim() || "{}");
  } catch {
    invocation = {};
  }
  const args = { ...(invocation.inputs || {}), ...(invocation.resolved_inputs || {}) };

  const respond = (status, output, stderr = "") => {
    process.stdout.write(
      JSON.stringify({
        schema: "runx.external_adapter.response.v1",
        protocol_version: "runx.external_adapter.v1",
        adapter_id: invocation.adapter_id,
        invocation_id: invocation.invocation_id,
        status,
        exit_code: status === "completed" ? 0 : 1,
        observed_at: "2026-06-02T00:00:00Z",
        stdout: JSON.stringify(output),
        stderr,
        output,
        artifacts: [],
        telemetry: []
      })
    );
  };

  let spec;
  try {
    spec = JSON.parse(readFileSync(new URL("./openapi.json", import.meta.url)));
  } catch (error) {
    respond("failed", { error: "cannot read openapi.json" }, String(error));
    return;
  }

  const wanted = args.operation_id || args.operationId;
  let found = null;
  for (const [path, methods] of Object.entries(spec.paths || {})) {
    for (const [method, op] of Object.entries(methods)) {
      if (op && op.operationId === wanted) {
        found = { path, method, op };
      }
    }
  }
  if (!found) {
    const available = Object.values(spec.paths || {})
      .flatMap((methods) => Object.values(methods))
      .map((op) => op && op.operationId)
      .filter(Boolean);
    respond(
      "failed",
      { error: `operation '${wanted}' not found`, available },
      `operation '${wanted}' not found; received inputs ${JSON.stringify(args)}`
    );
    return;
  }

  const params = found.op.parameters || [];
  const missing = [];
  let path = found.path;
  const query = [];
  for (const p of params) {
    const value = args[p.name];
    if (value === undefined || value === null) {
      if (p.required) missing.push(p.name);
      continue;
    }
    if (p.in === "path") {
      path = path.replace(`{${p.name}}`, encodeURIComponent(value));
    } else if (p.in === "query") {
      query.push(`${encodeURIComponent(p.name)}=${encodeURIComponent(value)}`);
    }
  }
  if (missing.length) {
    respond(
      "failed",
      { error: "missing required parameters", missing },
      `missing required parameters: ${missing.join(", ")}`
    );
    return;
  }

  const base = (spec.servers && spec.servers[0] && spec.servers[0].url) || "";
  const resolvedUrl = base + path + (query.length ? `?${query.join("&")}` : "");
  respond("completed", {
    ok: true,
    spec_title: spec.info && spec.info.title,
    operation_id: wanted,
    method: found.method.toUpperCase(),
    resolved_url: resolvedUrl,
    executed: false
  });
});
