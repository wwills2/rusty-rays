import { createRequire } from "node:module";
import type {bindings} from "./bindings/index.d.ts";

// type exports
export type Config = bindings.Config;
export type Tracer = bindings.Tracer;
export type Model = bindings.Model;
export type Sphere = bindings.Sphere;
export type Surface = bindings.Surface;
export type Color = bindings.Color;
export type Coords = bindings.Coords;
export type IntersectedObjectInfo = bindings.IntersectedObjectInfo

// code exports
const require = createRequire(import.meta.url);
type NapiBindings = typeof import("./bindings/index.d.ts").bindings;
const cjs = require("./bindings/index.cjs") as { bindings: NapiBindings };

export const {
    Model,
    Tracer,
    logWarn,
    logInfo,
    logError,
    logTrace,
    logDebug,
    shutdownLogger,
    getConfig,
    setConfig,
    getDefaultConfig,
} = cjs.bindings;
export default cjs.bindings;