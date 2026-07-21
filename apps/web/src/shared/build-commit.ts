declare const __ECHLUB_SOFTWARE_COMMIT__: string;

export function injectedSoftwareCommit(): string {
  if (typeof __ECHLUB_SOFTWARE_COMMIT__ === "string" && __ECHLUB_SOFTWARE_COMMIT__.length >= 7) {
    return __ECHLUB_SOFTWARE_COMMIT__;
  }
  return "";
}
