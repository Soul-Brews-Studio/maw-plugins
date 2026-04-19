// Thin re-export over maw-js/sdk. cmdBud is now exposed on the SDK surface
// as of maw-js#646 (SDK expansion: bud + oracle + transport). The plugin
// handler (./index.ts) imports cmdBud from here, keeping the layering
// identical to other SDK-clean plugins in this repo.
//
// Closes the residual standalone-bud half of Soul-Brews-Studio/maw-js#402.

export { cmdBud } from "maw-js/sdk";
export type { BudOpts } from "maw-js/sdk";
