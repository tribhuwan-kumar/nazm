import type { ItemMetaData } from "$lib/aria2/types";
import { defaultGlobalStat } from "$lib/aria2/client";

export type ConnectionState =
  | "disconnected"
  | "connecting"
  | "connected";

export const aria2State = $state({
  connectionState: "disconnected" as ConnectionState,

  globalStats: {
    ...defaultGlobalStat,
  },

  items: [] as ItemMetaData[],
});

export function resetGlobalStats() {
  aria2State.globalStats = {
    ...defaultGlobalStat,
  };
}
