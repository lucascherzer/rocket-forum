/**
 * Returns a consistent, light HSL background color for a given tag string.
 * The color is generated from a hash of the string, so the same tag always gets the same color.
 * @param tag - The string for which to generate a color.
 * @returns An HSL color string.
 */
export function getHashtagColor(tag: string): string {
    let hash = 0;
    for (let i = 0; i < tag.length; i++) {
        hash = ((hash << 5) - hash) + tag.charCodeAt(i);
        hash |= 0;
    }
    const hue = Math.abs(hash % 360);
    return `hsl(${hue}, 85%, 90%)`;
}

/**
 * Determines a suitable text color (black or white) for a given HSL background color,
 * based on its lightness value, to ensure good contrast and readability.
 * @param backgroundColor - The HSL background color string.
 * @returns A hex color string ('#000000' for black or '#ffffff' for white).
 */
export function getTextColor(backgroundColor: string): string {
    const hslMatch = backgroundColor.match(/hsl\((\d+),\s*(\d+)%,\s*(\d+)%\)/);
    if (!hslMatch) {
        return '#000000';
    }
    const hue = parseInt(hslMatch[1]);
    const saturation = parseInt(hslMatch[2]);
    const lightness = parseInt(hslMatch[3]);
    if (lightness > 60) {
        return '#000000';
    } else {
        return '#ffffff';
    }
}