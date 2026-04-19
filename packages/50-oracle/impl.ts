// Thin re-exports over maw-js/sdk. cmdOracle* are now on the SDK surface
// as of maw-js#646 (SDK expansion: bud + oracle + transport). Handler in
// ./index.ts imports these from here, mirroring the layering used by the
// other SDK-clean plugins in this repo.
//
// Closes the residual standalone-oracle half of Soul-Brews-Studio/maw-js#402.

export {
  cmdOracleList,
  cmdOracleScan,
  cmdOracleFleet,
  cmdOracleAbout,
} from "maw-js/sdk";
