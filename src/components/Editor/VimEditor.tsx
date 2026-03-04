import React, { useState, useEffect, useRef, useCallback } from 'react';

type VimMode = 'normal' | 'insert' | 'visual' | 'command';

interface VimEditorProps {
    filePath: string;
    initialContent: string;
    onSave: (content: string) => Promise<boolean>;
    onClose: () => void;
    onContentChange: (content: string) => void;
    hasUnsavedChanges: boolean;
}

const VimEditor: React.FC<VimEditorProps> = ({
    filePath,
    initialContent,
    onSave,
    onClose,
    onContentChange,
    hasUnsavedChanges
}) => {
    const [lines, setLines] = useState<string[]>(initialContent.split('\n'));
    const [cursorRow, setCursorRow] = useState(0);
    const [cursorCol, setCursorCol] = useState(0);
    const [mode, setMode] = useState<VimMode>('normal');
    const [commandInput, setCommandInput] = useState('');
    const [statusMessage, setStatusMessage] = useState('');
    const [visualStart, setVisualStart] = useState<{ row: number; col: number } | null>(null);
    const [yankBuffer, setYankBuffer] = useState<string>('');
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const commandInputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        if (mode === 'command' && commandInputRef.current) {
            commandInputRef.current.focus();
        } else if (textareaRef.current) {
            textareaRef.current.focus();
        }
    }, [mode]);

    useEffect(() => {
        const content = lines.join('\n');
        if (content !== initialContent) {
            onContentChange(content);
        }
    }, [lines]);

    const moveCursor = useCallback((row: number, col: number) => {
        const newRow = Math.max(0, Math.min(row, lines.length - 1));
        const newCol = Math.max(0, Math.min(col, lines[newRow].length));
        setCursorRow(newRow);
        setCursorCol(newCol);
    }, [lines]);

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (mode === 'command') return;

        // Insert mode
        if (mode === 'insert') {
            if (e.key === 'Escape') {
                e.preventDefault();
                setMode('normal');
                moveCursor(cursorRow, Math.max(0, cursorCol - 1));
                return;
            }
            return; // Let textarea handle other keys
        }

        // Normal mode
        if (mode === 'normal') {
            e.preventDefault();

            switch (e.key) {
                case 'i':
                    setMode('insert');
                    break;
                case 'I':
                    setMode('insert');
                    setCursorCol(0);
                    break;
                case 'a':
                    setMode('insert');
                    moveCursor(cursorRow, cursorCol + 1);
                    break;
                case 'A':
                    setMode('insert');
                    setCursorCol(lines[cursorRow].length);
                    break;
                case 'o':
                    setMode('insert');
                    const newLines = [...lines];
                    newLines.splice(cursorRow + 1, 0, '');
                    setLines(newLines);
                    moveCursor(cursorRow + 1, 0);
                    break;
                case 'O':
                    setMode('insert');
                    const newLinesAbove = [...lines];
                    newLinesAbove.splice(cursorRow, 0, '');
                    setLines(newLinesAbove);
                    setCursorCol(0);
                    break;
                case 'h':
                    moveCursor(cursorRow, cursorCol - 1);
                    break;
                case 'j':
                    moveCursor(cursorRow + 1, cursorCol);
                    break;
                case 'k':
                    moveCursor(cursorRow - 1, cursorCol);
                    break;
                case 'l':
                    moveCursor(cursorRow, cursorCol + 1);
                    break;
                case '0':
                    setCursorCol(0);
                    break;
                case '$':
                    setCursorCol(lines[cursorRow].length);
                    break;
                case 'g':
                    if (e.shiftKey) {
                        moveCursor(lines.length - 1, 0);
                    }
                    break;
                case 'x':
                    if (lines[cursorRow].length > 0) {
                        const newLines = [...lines];
                        newLines[cursorRow] = newLines[cursorRow].slice(0, cursorCol) + newLines[cursorRow].slice(cursorCol + 1);
                        setLines(newLines);
                    }
                    break;
                case 'd':
                    if (e.shiftKey) { // D - delete to end of line
                        const newLines = [...lines];
                        setYankBuffer(newLines[cursorRow].slice(cursorCol));
                        newLines[cursorRow] = newLines[cursorRow].slice(0, cursorCol);
                        setLines(newLines);
                    }
                    break;
                case 'y':
                    if (e.shiftKey) { // Y - yank line
                        setYankBuffer(lines[cursorRow]);
                        setStatusMessage('1 line yanked');
                        setTimeout(() => setStatusMessage(''), 2000);
                    }
                    break;
                case 'p':
                    if (yankBuffer) {
                        const newLines = [...lines];
                        newLines.splice(cursorRow + 1, 0, yankBuffer);
                        setLines(newLines);
                        moveCursor(cursorRow + 1, 0);
                    }
                    break;
                case 'u':
                    // TODO: Implement undo
                    setStatusMessage('Undo not yet implemented');
                    setTimeout(() => setStatusMessage(''), 2000);
                    break;
                case 'v':
                    setMode('visual');
                    setVisualStart({ row: cursorRow, col: cursorCol });
                    break;
                case ':':
                    setMode('command');
                    setCommandInput('');
                    break;
            }
        }

        // Visual mode
        if (mode === 'visual') {
            e.preventDefault();

            switch (e.key) {
                case 'Escape':
                    setMode('normal');
                    setVisualStart(null);
                    break;
                case 'h':
                    moveCursor(cursorRow, cursorCol - 1);
                    break;
                case 'j':
                    moveCursor(cursorRow + 1, cursorCol);
                    break;
                case 'k':
                    moveCursor(cursorRow - 1, cursorCol);
                    break;
                case 'l':
                    moveCursor(cursorRow, cursorCol + 1);
                    break;
                case 'y':
                    if (visualStart) {
                        const startRow = Math.min(visualStart.row, cursorRow);
                        const endRow = Math.max(visualStart.row, cursorRow);
                        const selectedLines = lines.slice(startRow, endRow + 1);
                        setYankBuffer(selectedLines.join('\n'));
                        setStatusMessage(`${selectedLines.length} line(s) yanked`);
                        setTimeout(() => setStatusMessage(''), 2000);
                        setMode('normal');
                        setVisualStart(null);
                    }
                    break;
                case 'd':
                    if (visualStart) {
                        const startRow = Math.min(visualStart.row, cursorRow);
                        const endRow = Math.max(visualStart.row, cursorRow);
                        const selectedLines = lines.slice(startRow, endRow + 1);
                        setYankBuffer(selectedLines.join('\n'));
                        const newLines = [...lines];
                        newLines.splice(startRow, endRow - startRow + 1);
                        setLines(newLines.length === 0 ? [''] : newLines);
                        moveCursor(startRow, 0);
                        setMode('normal');
                        setVisualStart(null);
                    }
                    break;
            }
        }
    };

    const handleTextareaChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        if (mode !== 'insert') return;
        const newLines = e.target.value.split('\n');
        setLines(newLines);
    };

    const handleCommandSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const cmd = commandInput.trim();

        if (cmd === 'w' || cmd === 'write') {
            const success = await onSave(lines.join('\n'));
            if (success) {
                setStatusMessage(`"${filePath}" written`);
                setTimeout(() => setStatusMessage(''), 2000);
            }
        } else if (cmd === 'q' || cmd === 'quit') {
            onClose();
        } else if (cmd === 'wq' || cmd === 'x') {
            const success = await onSave(lines.join('\n'));
            if (success) {
                onClose();
            }
        } else if (cmd === 'q!') {
            onClose();
        } else {
            setStatusMessage(`Unknown command: ${cmd}`);
            setTimeout(() => setStatusMessage(''), 2000);
        }

        setMode('normal');
        setCommandInput('');
    };

    const getModeDisplay = () => {
        switch (mode) {
            case 'insert': return '-- INSERT --';
            case 'visual': return '-- VISUAL --';
            case 'command': return '';
            default: return '';
        }
    };

    return (
        <div className="h-screen flex flex-col bg-gray-900 text-white font-mono">
            {/* Editor area */}
            <div className="flex-1 overflow-hidden relative">
                <textarea
                    ref={textareaRef}
                    value={lines.join('\n')}
                    onChange={handleTextareaChange}
                    onKeyDown={handleKeyDown}
                    className="w-full h-full bg-gray-900 text-white p-4 font-mono resize-none outline-none"
                    style={{
                        caretColor: mode === 'insert' ? 'white' : 'transparent',
                        lineHeight: '1.5rem'
                    }}
                    spellCheck={false}
                />
            </div>

            {/* Status bar */}
            <div className="bg-gray-800 px-4 py-1 flex justify-between items-center text-sm border-t border-gray-700">
                <div className="flex items-center space-x-4">
                    <span className="text-green-400">{getModeDisplay()}</span>
                    {statusMessage && <span className="text-yellow-400">{statusMessage}</span>}
                </div>
                <div className="flex items-center space-x-4">
                    {hasUnsavedChanges && <span className="text-red-400">[Modified]</span>}
                    <span>{filePath}</span>
                    <span>{cursorRow + 1},{cursorCol + 1}</span>
                    <span>{lines.length} lines</span>
                </div>
            </div>

            {/* Command line */}
            {mode === 'command' && (
                <form onSubmit={handleCommandSubmit} className="bg-gray-900 px-4 py-1 border-t border-gray-700">
                    <span className="text-white mr-2">:</span>
                    <input
                        ref={commandInputRef}
                        type="text"
                        value={commandInput}
                        onChange={(e) => setCommandInput(e.target.value)}
                        onKeyDown={(e) => {
                            if (e.key === 'Escape') {
                                e.preventDefault();
                                setMode('normal');
                                setCommandInput('');
                            }
                        }}
                        className="bg-transparent text-white outline-none flex-1"
                        autoFocus
                    />
                </form>
            )}
        </div>
    );
};

export default VimEditor;
