import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import TextEditor from '../components/Editor/TextEditor';
import VimEditor from '../components/Editor/VimEditor';
import NanoEditor from '../components/Editor/NanoEditor';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn()
}));

describe('TextEditor', () => {
    const mockOnClose = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should render loading state initially', () => {
        const { invoke } = require('@tauri-apps/api/core');
        invoke.mockResolvedValue('test content');

        render(
            <TextEditor
                filePath="/test/file.txt"
                onClose={mockOnClose}
                mode="nano"
            />
        );

        expect(screen.getByText(/Loading/i)).toBeInTheDocument();
    });

    it('should load file content', async () => {
        const { invoke } = require('@tauri-apps/api/core');
        invoke.mockResolvedValue('test content');

        render(
            <TextEditor
                filePath="/test/file.txt"
                onClose={mockOnClose}
                mode="nano"
            />
        );

        await waitFor(() => {
            expect(invoke).toHaveBeenCalledWith('read_file_for_editor', {
                path: '/test/file.txt'
            });
        });
    });

    it('should handle file not found error', async () => {
        const { invoke } = require('@tauri-apps/api/core');
        invoke.mockRejectedValue(new Error('No such file'));

        render(
            <TextEditor
                filePath="/test/nonexistent.txt"
                onClose={mockOnClose}
                mode="nano"
            />
        );

        // Should still render editor with empty content
        await waitFor(() => {
            expect(invoke).toHaveBeenCalled();
        });
    });
});

describe('NanoEditor', () => {
    const mockOnSave = vi.fn();
    const mockOnClose = vi.fn();
    const mockOnContentChange = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should render with initial content', () => {
        render(
            <NanoEditor
                filePath="/test/file.txt"
                initialContent="Hello World"
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={false}
            />
        );

        expect(screen.getByText('GNU nano')).toBeInTheDocument();
        expect(screen.getByText('/test/file.txt')).toBeInTheDocument();
    });

    it('should show modified indicator when content changes', () => {
        render(
            <NanoEditor
                filePath="/test/file.txt"
                initialContent="Hello World"
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={true}
            />
        );

        expect(screen.getByText('[Modified]')).toBeInTheDocument();
    });

    it('should display keyboard shortcuts', () => {
        render(
            <NanoEditor
                filePath="/test/file.txt"
                initialContent=""
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={false}
            />
        );

        expect(screen.getByText(/Help/i)).toBeInTheDocument();
        expect(screen.getByText(/Save/i)).toBeInTheDocument();
        expect(screen.getByText(/Exit/i)).toBeInTheDocument();
    });

    it('should call onContentChange when text changes', () => {
        render(
            <NanoEditor
                filePath="/test/file.txt"
                initialContent="Hello"
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={false}
            />
        );

        const textarea = screen.getByRole('textbox');
        fireEvent.change(textarea, { target: { value: 'Hello World' } });

        expect(mockOnContentChange).toHaveBeenCalledWith('Hello World');
    });
});

describe('VimEditor', () => {
    const mockOnSave = vi.fn();
    const mockOnClose = vi.fn();
    const mockOnContentChange = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should render with initial content', () => {
        render(
            <VimEditor
                filePath="/test/file.txt"
                initialContent="Hello World"
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={false}
            />
        );

        expect(screen.getByText('/test/file.txt')).toBeInTheDocument();
    });

    it('should start in normal mode', () => {
        render(
            <VimEditor
                filePath="/test/file.txt"
                initialContent="Hello"
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={false}
            />
        );

        // Normal mode should not show mode indicator
        expect(screen.queryByText('-- INSERT --')).not.toBeInTheDocument();
    });

    it('should show modified indicator', () => {
        render(
            <VimEditor
                filePath="/test/file.txt"
                initialContent="Hello"
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={true}
            />
        );

        expect(screen.getByText('[Modified]')).toBeInTheDocument();
    });

    it('should display line count', () => {
        render(
            <VimEditor
                filePath="/test/file.txt"
                initialContent="Line 1\nLine 2\nLine 3"
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={false}
            />
        );

        expect(screen.getByText(/3 lines/i)).toBeInTheDocument();
    });

    it('should call onContentChange when content changes', () => {
        render(
            <VimEditor
                filePath="/test/file.txt"
                initialContent="Hello"
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={false}
            />
        );

        const textarea = screen.getByRole('textbox');
        fireEvent.change(textarea, { target: { value: 'Hello World' } });

        expect(mockOnContentChange).toHaveBeenCalled();
    });
});

describe('Editor Integration', () => {
    it('should handle save operation in nano', async () => {
        const mockOnSave = vi.fn().mockResolvedValue(true);
        const mockOnClose = vi.fn();
        const mockOnContentChange = vi.fn();

        render(
            <NanoEditor
                filePath="/test/file.txt"
                initialContent="Hello"
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={true}
            />
        );

        const textarea = screen.getByRole('textbox');
        
        // Simulate Ctrl+O (save)
        fireEvent.keyDown(textarea, { key: 'o', ctrlKey: true });

        await waitFor(() => {
            expect(mockOnSave).toHaveBeenCalledWith('Hello');
        });
    });

    it('should handle exit with unsaved changes', () => {
        const mockOnSave = vi.fn();
        const mockOnClose = vi.fn();
        const mockOnContentChange = vi.fn();
        
        // Mock window.confirm
        global.confirm = vi.fn(() => false);

        render(
            <NanoEditor
                filePath="/test/file.txt"
                initialContent="Hello"
                onSave={mockOnSave}
                onClose={mockOnClose}
                onContentChange={mockOnContentChange}
                hasUnsavedChanges={true}
            />
        );

        const textarea = screen.getByRole('textbox');
        
        // Simulate Ctrl+X (exit)
        fireEvent.keyDown(textarea, { key: 'x', ctrlKey: true });

        expect(global.confirm).toHaveBeenCalled();
    });
});
