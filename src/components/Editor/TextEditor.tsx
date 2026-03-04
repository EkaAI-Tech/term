import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import VimEditor from './VimEditor';
import NanoEditor from './NanoEditor';

export type EditorMode = 'vim' | 'nano';

interface TextEditorProps {
    filePath: string;
    onClose: () => void;
    mode: EditorMode;
}

const TextEditor: React.FC<TextEditorProps> = ({ filePath, onClose, mode }) => {
    const [content, setContent] = useState<string>('');
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);

    useEffect(() => {
        loadFile();
    }, [filePath]);

    const loadFile = async () => {
        try {
            setLoading(true);
            setError(null);
            const fileContent = await invoke<string>('read_file_for_editor', { path: filePath });
            setContent(fileContent);
            setHasUnsavedChanges(false);
        } catch (err: any) {
            // If file doesn't exist, start with empty content
            if (err.toString().includes('No such file')) {
                setContent('');
                setHasUnsavedChanges(false);
            } else {
                setError(`Failed to load file: ${err}`);
            }
        } finally {
            setLoading(false);
        }
    };

    const saveFile = async (contentToSave: string) => {
        try {
            await invoke('write_file_from_editor', {
                path: filePath,
                content: contentToSave
            });
            setHasUnsavedChanges(false);
            return true;
        } catch (err: any) {
            setError(`Failed to save file: ${err}`);
            return false;
        }
    };

    const handleContentChange = (newContent: string) => {
        if (newContent !== content) {
            setHasUnsavedChanges(true);
        }
        setContent(newContent);
    };

    const handleClose = () => {
        onClose();
    };

    if (loading) {
        return (
            <div className="fixed inset-0 bg-gray-900 flex items-center justify-center z-50">
                <div className="text-white text-xl">Loading {filePath}...</div>
            </div>
        );
    }

    if (error && !content) {
        return (
            <div className="fixed inset-0 bg-gray-900 flex items-center justify-center z-50">
                <div className="text-center">
                    <div className="text-red-400 text-xl mb-4">{error}</div>
                    <button
                        onClick={onClose}
                        className="bg-gray-700 hover:bg-gray-600 text-white px-4 py-2 rounded"
                    >
                        Close
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div className="fixed inset-0 bg-gray-900 z-50">
            {mode === 'vim' ? (
                <VimEditor
                    filePath={filePath}
                    initialContent={content}
                    onSave={saveFile}
                    onClose={handleClose}
                    onContentChange={handleContentChange}
                    hasUnsavedChanges={hasUnsavedChanges}
                />
            ) : (
                <NanoEditor
                    filePath={filePath}
                    initialContent={content}
                    onSave={saveFile}
                    onClose={handleClose}
                    onContentChange={handleContentChange}
                    hasUnsavedChanges={hasUnsavedChanges}
                />
            )}
        </div>
    );
};

export default TextEditor;
