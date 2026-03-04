import React, { useState, useEffect, useRef } from 'react';

interface NanoEditorProps {
    filePath: string;
    initialContent: string;
    onSave: (content: string) => Promise<boolean>;
    onClose: () => void;
    onContentChange: (content: string) => void;
    hasUnsavedChanges: boolean;
}

const NanoEditor: React.FC<NanoEditorProps> = ({
    filePath,
    initialContent,
    onSave,
    onClose,
    onContentChange,
    hasUnsavedChanges
}) => {
    const [content, setContent] = useState(initialContent);
    const [statusMessage, setStatusMessage] = useState('');
    const [showHelp, setShowHelp] = useState(false);
    const [cursorPosition, setCursorPosition] = useState({ line: 1, col: 1 });
    const textareaRef = useRef<HTMLTextAreaElement>(null);

    useEffect(() => {
        if (textareaRef.current) {
            textareaRef.current.focus();
        }
    }, []);

    useEffect(() => {
        if (content !== initialContent) {
            onContentChange(content);
        }
    }, [content]);

    const handleKeyDown = async (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        // Ctrl+S - Save
        if (e.ctrlKey && e.key === 's') {
            e.preventDefault();
            const success = await onSave(content);
            if (success) {
                setStatusMessage(`[ Wrote ${content.split('\n').length} lines ]`);
                setTimeout(() => setStatusMessage(''), 2000);
            }
        }
        // Ctrl+X - Exit
        else if (e.ctrlKey && e.key === 'x') {
            e.preventDefault();
            if (hasUnsavedChanges) {
                const confirmExit = window.confirm('Save modified buffer?');
                if (confirmExit) {
                    const success = await onSave(content);
                    if (success) {
                        onClose();
                    }
                } else {
                    onClose();
                }
            } else {
                onClose();
            }
        }
        // Ctrl+G - Help
        else if (e.ctrlKey && e.key === 'g') {
            e.preventDefault();
            setShowHelp(!showHelp);
        }
        // Ctrl+K - Cut line
        else if (e.ctrlKey && e.key === 'k') {
            e.preventDefault();
            const textarea = textareaRef.current;
            if (!textarea) return;

            const start = textarea.selectionStart;
            const lines = content.split('\n');
            let currentPos = 0;
            let lineIndex = 0;

            for (let i = 0; i < lines.length; i++) {
                if (currentPos + lines[i].length >= start) {
                    lineIndex = i;
                    break;
                }
                currentPos += lines[i].length + 1;
            }

            const newLines = [...lines];
            newLines.splice(lineIndex, 1);
            setContent(newLines.join('\n'));
            setStatusMessage('[ Cut line ]');
            setTimeout(() => setStatusMessage(''), 2000);
        }
        // Ctrl+W - Search
        else if (e.ctrlKey && e.key === 'w') {
            e.preventDefault();
            const searchTerm = prompt('Search:');
            if (searchTerm) {
                const index = content.indexOf(searchTerm);
                if (index !== -1 && textareaRef.current) {
                    textareaRef.current.setSelectionRange(index, index + searchTerm.length);
                    textareaRef.current.focus();
                } else {
                    setStatusMessage(`[ "${searchTerm}" not found ]`);
                    setTimeout(() => setStatusMessage(''), 2000);
                }
            }
        }
    };

    const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        setContent(e.target.value);
        updateCursorPosition(e.target);
    };

    const handleClick = (e: React.MouseEvent<HTMLTextAreaElement>) => {
        updateCursorPosition(e.currentTarget);
    };

    const handleKeyUp = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        updateCursorPosition(e.currentTarget);
    };

    const updateCursorPosition = (textarea: HTMLTextAreaElement) => {
        const text = textarea.value.substring(0, textarea.selectionStart);
        const lines = text.split('\n');
        const line = lines.length;
        const col = lines[lines.length - 1].length + 1;
        setCursorPosition({ line, col });
    };

    return (
        <div className="h-screen flex flex-col bg-gray-900 text-white font-mono">
            {/* Header */}
            <div className="bg-gray-800 px-4 py-2 border-b border-gray-700">
                <div className="flex justify-between items-center">
                    <span className="text-green-400 font-bold">GNU nano</span>
                    <span className="text-gray-400">{filePath}</span>
                    {hasUnsavedChanges && <span className="text-red-400">[Modified]</span>}
                </div>
            </div>

            {/* Editor area */}
            <div className="flex-1 overflow-hidden">
                <textarea
                    ref={textareaRef}
                    value={content}
                    onChange={handleChange}
                    onClick={handleClick}
                    onKeyUp={handleKeyUp}
                    onKeyDown={handleKeyDown}
                    className="w-full h-full bg-gray-900 text-white p-4 font-mono resize-none outline-none"
                    style={{ lineHeight: '1.5rem' }}
                    spellCheck={false}
                />
            </div>

            {/* Status message */}
            {statusMessage && (
                <div className="bg-gray-800 px-4 py-1 text-yellow-400 text-sm border-t border-gray-700">
                    {statusMessage}
                </div>
            )}

            {/* Help panel */}
            {showHelp && (
                <div className="bg-gray-800 px-4 py-2 border-t border-gray-700 text-sm">
                    <div className="grid grid-cols-2 gap-2">
                        <div><span className="text-cyan-400">^G</span> Get Help</div>
                        <div><span className="text-cyan-400">^S</span> Save File</div>
                        <div><span className="text-cyan-400">^W</span> Where Is (Search)</div>
                        <div><span className="text-cyan-400">^K</span> Cut Line</div>
                        <div><span className="text-cyan-400">^X</span> Exit</div>
                    </div>
                </div>
            )}

            {/* Bottom bar with shortcuts */}
            <div className="bg-gray-800 px-4 py-2 border-t border-gray-700">
                <div className="flex justify-between items-center text-xs">
                    <div className="flex space-x-4">
                        <span><span className="text-cyan-400">^G</span> Help</span>
                        <span><span className="text-cyan-400">^S</span> Save</span>
                        <span><span className="text-cyan-400">^W</span> Search</span>
                        <span><span className="text-cyan-400">^K</span> Cut</span>
                        <span><span className="text-cyan-400">^X</span> Exit</span>
                    </div>
                    <div className="text-gray-400">
                        Line {cursorPosition.line}, Col {cursorPosition.col}
                    </div>
                </div>
            </div>
        </div>
    );
};

export default NanoEditor;
