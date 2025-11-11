import path from "node:path";

export function adomPathOverride() {
  return (
    process.env.ADOM_EXECUTABLE ??
    path.join(process.cwd(), "..", "..", "adom-rs", "target", "debug", "adom")
  );
}
