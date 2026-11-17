/** Text direction, for lyrics in right-to-left scripts (Hebrew; Arabic's ranges are here too). */

const RTL = "\u0590-\u08FF\uFB1D-\uFDFF\uFE70-\uFEFC";
const STARTS_RTL = new RegExp(`^[^\\p{L}]*[${RTL}]`, "u");
/** right-to-left mark: an invisible character that reads as a right-to-left letter */
const RLM = "\u200F";

/** `text` reads right to left: its first letter is from a right-to-left script. */
export const isRtl = (text: string): boolean => STARTS_RTL.test(text);

/**
 * A syllable whose word goes on: a hyphen after it, in reading order. After right-to-left text a
 * mark follows the hyphen, or it is drawn on the right, where the word began.
 */
export const goesOn = (text: string): string => (isRtl(text) ? `${text}-${RLM}` : `${text}-`);
