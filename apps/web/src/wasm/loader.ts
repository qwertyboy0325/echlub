import init, { EchlubCore } from "../../../../crates/echlub-web/pkg/echlub_web.js";

export type EchlubCoreFacade = EchlubCore;

export async function loadEchlubCore(): Promise<typeof EchlubCore> {
  await init();
  return EchlubCore;
}
