export function getHashtagColor(tag: string): string {
    // Simple hash function for strings
    let hash = 0;
    for (let i = 0; i < tag.length; i++) {
        hash = ((hash << 5) - hash) + tag.charCodeAt(i);
        hash |= 0; // Convert to 32bit integer
    }
    
    // Create a hue value from the hash (0-360)
    const hue = Math.abs(hash % 360);
    
    // Use HSL to ensure good contrast with text
    // Fixed saturation and lightness for consistency
    return `hsl(${hue}, 85%, 90%)`;
}

export function getTextColor(backgroundColor: string): string {
    // Extract HSL values from the background color string
    const hslMatch = backgroundColor.match(/hsl\((\d+),\s*(\d+)%,\s*(\d+)%\)/);
    
    if (!hslMatch) {
        return '#000000'; // Default to black if parsing fails
    }
    
    const hue = parseInt(hslMatch[1]);
    const saturation = parseInt(hslMatch[2]);
    const lightness = parseInt(hslMatch[3]);
    
    // For HSL colors, we primarily base text contrast on the lightness value
    // Generally, if lightness > 60%, use black text; otherwise use white
    // Adjusted for certain hue ranges that may affect perceived brightness
    if (lightness > 60) {
        return '#000000'; // Black text for light backgrounds
    } else {
        return '#ffffff'; // White text for dark backgrounds
    }
}