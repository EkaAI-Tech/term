import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { parseInput } from '../../utils/intentParser';
import type { CommandProcessorProps } from './types';
import type { EditorMode } from '../Editor/TextEditor';

const useCommandProcessor = ({
    input,
    setInput,
    appendHistory,
    setIsProcessing,
    clearTerminal
}: CommandProcessorProps) => {
    const [showPasswordDialog, setShowPasswordDialog] = useState(false);
    const [pendingSudoCommand, setPendingSudoCommand] = useState('');
    const [editorFile, setEditorFile] = useState<string | null>(null);
    const [editorMode, setEditorMode] = useState<EditorMode>('nano');

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!input.trim()) return;

        appendHistory({ type: 'command', content: input });
        setIsProcessing(true);

        if (input.trim().toLowerCase() === 'clear') {
            clearTerminal();
            setInput("");
            setIsProcessing(false);
            return;
        }

        // Handle exit command
        if (input.trim().toLowerCase() === 'exit') {
            try {
                await getCurrentWindow().close();
            } catch (error) {
                appendHistory({
                    type: 'error',
                    content: `Failed to exit: ${error}`
                });
            } finally {
                setIsProcessing(false);
            }
            return;
        }

        // Handle API key related commands - redirect to settings UI
        if (input.trim() === 'apikey' || input.trim() === 'api-key' || 
            input.trim().startsWith('setapikey') || input.trim() === 'resetapikey') {
            appendHistory({
                content: 'API key management has been moved to the Settings UI.',
                type: 'output'
            });
            setInput('');
            setIsProcessing(false);
            return;
        }

        const parsed = parseInput(input);

        try {
            // Handle nano/vim/nvim commands
            if (input.trim().startsWith('nano ') || input.trim().startsWith('vim ') || 
                input.trim().startsWith('vi ') || input.trim().startsWith('nvim ')) {
                const parts = input.trim().split(' ');
                const command = parts[0];
                const filename = parts.slice(1).join(' ').trim();

                if (!filename) {
                    appendHistory({
                        type: 'error',
                        content: `Usage: ${command} <filename>`
                    });
                    setInput('');
                    setIsProcessing(false);
                    return;
                }

                setEditorMode(command === 'nano' ? 'nano' : 'vim');
                setEditorFile(filename);
                setInput('');
                setIsProcessing(false);
                return;
            }

            if (parsed.type === "system_command") {
                // Handle cd command separately
                if (parsed.command.trim().startsWith("cd ")) {
                    const path = parsed.command.trim().substring(3);
                    try {
                        const newDirectory = await invoke<string>("change_directory", { path });
                        appendHistory({
                            type: 'output',
                            content: `Changed directory to: ${newDirectory}`
                        });
                    } catch (error: any) {
                        appendHistory({
                            type: 'error',
                            content: error.toString()
                        });
                    }
                }
                else if (parsed.command.trim().startsWith("sudo ")) {
                    setPendingSudoCommand(parsed.command);
                    setShowPasswordDialog(true);
                    setInput("");
                    return;
                }
                else {
                    const result = await invoke<string>("run_shell", { command: parsed.command });
                    const commandNotFoundRegex = /\bcommand not found\b/i;
                    const notRecognizedRegex = /\bnot recognized\b/i;

                    if (commandNotFoundRegex.test(result) || notRecognizedRegex.test(result)) {
                        appendHistory({ type: 'error', content: result.trim() });
                        const fallback = await invoke<string>("ask_llm", { prompt: input });
                        appendHistory({ type: 'llm', content: fallback.trim() });
                    } else {
                        appendHistory({ type: 'output', content: result.trim() });
                    }
                }
            } else if (parsed.type === "file_view") {
                const result = await invoke<string>("read_file", { path: parsed.filename });
                appendHistory({ type: 'output', content: result.trim() });
            } else if (parsed.type === "file_summary") {
                const fileContent = await invoke<string>("read_file", { path: parsed.filename });
                const summary = await invoke<string>("ask_llm", {
                    prompt: `Summarize the following file:\n\n${fileContent}`,
                });
                appendHistory({ type: 'llm', content: summary.trim() });
            } else if (parsed.type === "llm_query") {
                try {
                    const result = await invoke<string>("ask_llm", { prompt: parsed.prompt });
                    appendHistory({ type: 'llm', content: result.trim() });
                } catch (error: any) {
                    // Handle API configuration errors
                    if (error.toString().includes("not configured") || error.toString().includes("No default")) {
                        appendHistory({
                            type: 'error',
                            content: 'No AI provider configured. Click the settings button (⚙️) in the top-right corner to configure your AI provider.'
                        });
                    } else {
                        appendHistory({ type: 'error', content: `${error.message || error}` });
                    }
                }
            } else {
                try {
                    const fallback = await invoke<string>("ask_llm", { prompt: input });
                    appendHistory({ type: 'llm', content: fallback.trim() });
                } catch (error: any) {
                    // Handle API configuration errors
                    if (error.toString().includes("not configured") || error.toString().includes("No default")) {
                        appendHistory({
                            type: 'error',
                            content: 'No AI provider configured. Click the settings button (⚙️) in the top-right corner to configure your AI provider.'
                        });
                    } else {
                        appendHistory({ type: 'error', content: `${error.message || error}` });
                    }
                }
            }

            appendHistory({ type: 'separator', content: '' });
        } catch (err: any) {
            appendHistory({ type: 'error', content: `${err.message || err}` });
            appendHistory({ type: 'separator', content: '' });
        } finally {
            setIsProcessing(false);
        }

        setInput("");
    };

    const handleSudoWithPassword = async (password: string) => {
        setShowPasswordDialog(false);

        appendHistory({ type: 'output', content: "⚡ Executing sudo command... This may take a moment." });

        try {
            const result = await invoke<string>("run_sudo_command", {
                command: pendingSudoCommand,
                password: password
            });
            appendHistory({ type: 'output', content: result.trim() });
        } catch (err: any) {
            let errorMessage = `${err.message || err}`;

            console.error("Sudo error:", errorMessage);

            if (errorMessage.includes("Incorrect password")) {
                appendHistory({
                    type: 'error',
                    content: "🔒 Authentication failed: Incorrect password provided."
                });
            } else {
                appendHistory({
                    type: 'error',
                    content: `Error executing sudo command: ${errorMessage}`
                });
            }
        } finally {
            appendHistory({ type: 'separator', content: '' });
            setIsProcessing(false);
            setPendingSudoCommand('');
        }
    };

    return {
        handleSubmit,
        handleSudoWithPassword,
        showPasswordDialog,
        setShowPasswordDialog,
        pendingSudoCommand,
        setPendingSudoCommand,
        editorFile,
        editorMode,
        setEditorFile
    };
};

export default useCommandProcessor;
