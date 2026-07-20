import { BrkClient } from "../modules/brk-client/index.js";

// const brk = new BrkClient("https://dogeview.space");
const brk = new BrkClient("/");

console.log(`VERSION = ${brk.VERSION}`);

export { brk };
