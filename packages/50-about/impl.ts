// Thin re-export over maw-js/sdk. cmdOracleAbout is now on the SDK surface
// as of maw-js#646 (SDK expansion: bud + oracle + transport). Handler in
// ./index.ts imports this from here, mirroring the layering used by the
// other SDK-clean plugins in this repo.
//
// Closes the residual standalone-about half of Soul-Brews-Studio/maw-js#402.

export { cmdOracleAbout } from "maw-js/sdk";
