export type Intent =
    | { type: 'system_command'; command: string }
    | { type: 'llm_query'; prompt: string }
    | { type: 'file_view'; filename: string }
    | { type: 'file_summary'; filename: string }
    | { type: 'unknown'; input: string };

export function parseInput(input: string): Intent {
    const trimmed = input.trim();

    // File viewing: "cat abc.txt", "view abc.txt", "cat filename"
    if (/^(view|show|cat)\s+[\w./-]+/i.test(trimmed)) {
        const parts = trimmed.split(/\s+/);
        if (parts.length >= 2) {
            const filename = parts.slice(1).join(' ');
            return { type: 'file_view', filename };
        }
    }

    // File summary: "summarize abc.txt", "give summary of abc.txt"
    if (/summar(y|ise|ize).+/i.test(trimmed)) {
        const match = trimmed.match(/(?:of|about|on)\s+([\w./-]+)/i);
        if (match) return { type: 'file_summary', filename: match[1] };
    }

    // Likely system command: no question form, looks like shell input
    if (/^[a-zA-Z0-9./_-]+(\s+.*)?$/.test(trimmed)) {
        if (trimmed.startsWith('cd ')) {
            return { type: 'system_command', command: trimmed };
        }
        return { type: 'system_command', command: trimmed };
    }

    // Fallback: natural language
    if (trimmed.length > 0) {
        return { type: 'llm_query', prompt: trimmed };
    }

    return { type: 'unknown', input: input };
}
