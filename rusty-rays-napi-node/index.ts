import {createRequire} from "node:module";

const require = createRequire(import.meta.url);

type NapiBindings = typeof import("./bindings/index.d.ts").bindings;
const cjs = require("./bindings/index.cjs") as { bindings: NapiBindings };

export const {Model, Tracer, logWarn, logInfo, logError, logTrace, logDebug, shutdownLogger} = cjs.bindings;
export default cjs.bindings;