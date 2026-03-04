import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import TerminalOutput from './TerminalOutput';
import TerminalInput from './TerminalInput';
import PasswordDialog from '../PasswordDialog';
import Settings from '../Settings';
import TextEditor from '../Editor/TextEditor';
import useTerminal from '../../hooks/useTerminal';
import useCommandProcessor from './CommandProcessor';

const Terminal: React.FC = () => {
    const {
        input,
        setInput,
        history,
        isProcessing,
        setIsProcessing,
        appendHistory,
        clearTerminal
    } = useTerminal();

    const {
        handleSubmit,
        handleSudoWithPassword,
        showPasswordDialog,
        setShowPasswordDialog,
        pendingSudoCommand,
        editorFile,
        editorMode,
        setEditorFile
    } = useCommandProcessor({
        input,
        setInput,
        appendHistory,
        setIsProcessing,
        clearTerminal
    });

    const [showSettings, setShowSettings] = useState(false);

    // Check if AI settings exist on startup
    useEffect(() => {
        let mounted = true;

        const checkSettings = async () => {
            try {
                const defaultProvider = await invoke<[string, { api_key: string; model: string; is_default: boolean }] | null>('get_default_provider');
                if (!defaultProvider && mounted) {
                    appendHistory({
                        type: 'output',
                        content: 'No AI provider configured. Click the settings button (⚙️) in the top-right corner to configure your AI provider.'
                    });
                }
            } catch (error) {
                console.error('Failed to check AI settings:', error);
            }
        };

        checkSettings();

        return () => {
            mounted = false;
        };
    }, []);

    return (
        <>
            {editorFile ? (
                <TextEditor
                    filePath={editorFile}
                    onClose={() => setEditorFile(null)}
                    mode={editorMode || 'nano'}
                />
            ) : (
                <div className="flex flex-col h-screen w-screen bg-gray-900 text-white relative">
                    {/* Settings Button */}
                    <button
                        onClick={() => setShowSettings(true)}
                        className="absolute top-4 right-4 z-10 bg-gray-800 hover:bg-gray-700 text-white p-3 rounded-full shadow-lg transition-colors"
                        title="AI Settings"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            className="h-6 w-6"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                            />
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                            />
                        </svg>
                    </button>

                    <TerminalOutput history={history} isProcessing={isProcessing} />

                    <TerminalInput
                        input={input}
                        setInput={setInput}
                        isProcessing={isProcessing}
                        onSubmit={handleSubmit}
                    />

                    <PasswordDialog
                        isOpen={showPasswordDialog}
                        onClose={() => {
                            setShowPasswordDialog(false);
                            setIsProcessing(false);
                        }}
                        onSubmit={handleSudoWithPassword}
                        commandText={pendingSudoCommand}
                    />

                    <Settings
                        isOpen={showSettings}
                        onClose={() => setShowSettings(false)}
                    />
                </div>
            )}
        </>
    );
};

export default Terminal;
