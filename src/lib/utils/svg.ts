/**
 * Converts SVG transform attributes to CSS transform properties
 * @param svgTransform SVG transform attribute value
 * @returns CSS transform property value
 */
export function svgToCssTransform(svgTransform: string): string {
  return svgTransform
    .replace(/translate\(([-\d.,]+),([-\d.,]+)\)/, (_, x, y) => `translate(${x}px, ${y}px)`)
    .replace(/scale\(([-\d.,]+)\)/, (_, s) => `scale(${s})`);
}
