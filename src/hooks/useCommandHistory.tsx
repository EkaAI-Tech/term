import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

const useCommandHistory = () => {
    const [commandHistory, setCommandHistory] = useState<string[]>([]);
    const [historyIndex, setHistoryIndex] = useState(-1);
    const [isLoaded, setIsLoaded] = useState(false);

    // Load history from file on mount
    useEffect(() => {
        const loadHistory = async () => {
            try {
                const history = await invoke<string[]>('load_command_history');
                setCommandHistory(history);
                setIsLoaded(true);
            } catch (error) {
                console.error('Failed to load command history:', error);
                setIsLoaded(true);
            }
        };

        loadHistory();
    }, []);

    const addToHistory = async (command: string) => {
        // Add to local state
        setCommandHistory(prev => [...prev, command]);
        setHistoryIndex(-1);

        // Save to file
        try {
            await invoke('save_command_to_history', { command });
        } catch (error) {
            console.error('Failed to save command to history:', error);
        }
    };

    const navigateHistory = (direction: 'up' | 'down'): string | undefined => {
        if (direction === 'up') {
            if (commandHistory.length > 0 && historyIndex < commandHistory.length - 1) {
                const newIndex = historyIndex + 1;
                setHistoryIndex(newIndex);
                return commandHistory[commandHistory.length - 1 - newIndex];
            }
        } else if (direction === 'down') {
            if (historyIndex > 0) {
                const newIndex = historyIndex - 1;
                setHistoryIndex(newIndex);
                return commandHistory[commandHistory.length - 1 - newIndex];
            } else if (historyIndex === 0) {
                setHistoryIndex(-1);
                return '';
            }
        }
        return undefined;
    };

    const clearHistory = async () => {
        setCommandHistory([]);
        setHistoryIndex(-1);

        try {
            await invoke('clear_command_history');
        } catch (error) {
            console.error('Failed to clear command history:', error);
        }
    };

    return {
        commandHistory,
        historyIndex,
        isLoaded,
        addToHistory,
        navigateHistory,
        clearHistory
    };
};

export default useCommandHistory;
