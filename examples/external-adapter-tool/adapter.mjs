// Minimal external adapter: echoes its inputs. The runx.external_adapter.v1
// protocol frame is handled by the shared adapter kit; this file is just the
// adapter's logic.
import { runAdapter } from "../adapter-kit/adapter.mjs";

runAdapter(({ inputs }) => ({ ok: true, inputs }));
