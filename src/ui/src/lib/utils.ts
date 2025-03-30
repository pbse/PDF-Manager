/**
 * Parses a string like "1, 3-5, 8" into an array of numbers [1, 3, 4, 5, 8].
 * Returns null if the format is invalid.
 */
export function parsePageString(pageString: string): number[] | null {
    const result: Set<number> = new Set();
    if (!pageString.trim()) {
        return []; // Treat empty string as empty array
    }

    const parts = pageString.split(',');

    for (const part of parts) {
        const trimmedPart = part.trim();
        if (!trimmedPart) continue;

        if (trimmedPart.includes('-')) {
            const rangeParts = trimmedPart.split('-');
            if (rangeParts.length !== 2) return null; // Invalid range format

            const startStr = rangeParts[0].trim();
            const endStr = rangeParts[1].trim();

            if (!/^\d+$/.test(startStr) || !/^\d+$/.test(endStr)) return null; // Not numbers

            const start = parseInt(startStr, 10);
            const end = parseInt(endStr, 10);

            if (isNaN(start) || isNaN(end) || start <= 0 || end <= 0 || start > end) {
                return null; // Invalid numbers or range order
            }

            for (let i = start; i <= end; i++) {
                result.add(i);
            }
        } else {
            if (!/^\d+$/.test(trimmedPart)) return null; // Not a number
            const pageNum = parseInt(trimmedPart, 10);
            if (isNaN(pageNum) || pageNum <= 0) return null; // Invalid number
            result.add(pageNum);
        }
    }

    // Sort the unique page numbers
    return Array.from(result).sort((a, b) => a - b);
}