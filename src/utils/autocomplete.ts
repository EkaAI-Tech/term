import { invoke } from '@tauri-apps/api/core';

// Common shell commands for autocomplete
const COMMON_COMMANDS = [
    'ls', 'cd', 'mkdir', 'rmdir', 'touch', 'rm', 'cp', 'mv',
    'cat', 'echo', 'grep', 'find', 'ps', 'kill', 'sudo',
    'chmod', 'chown', 'df', 'du', 'tar', 'zip', 'unzip',
    'ssh', 'scp', 'ping', 'wget', 'curl', 'apt', 'apt-get',
    'yum', 'dnf', 'pacman', 'brew', 'git', 'npm', 'yarn',
    'node', 'python', 'python3', 'pip', 'pip3', 'java', 'javac', 'jar',
    'docker', 'kubectl', 'systemctl', 'journalctl', 'clear', 'history',
    'man', 'help', 'exit', 'perl', 'ruby', 'go', 'rustc', 'cargo',
    'make', 'cmake', 'gcc', 'g++', 'clang', 'nvim', 'vim', 'nano',
    'top', 'htop', 'screen', 'tmux', 'service', 'ifconfig', 'ip',
    'whoami', 'env', 'export', 'alias', 'unalias', 'which', 'whereis',
    'locate', 'mount', 'umount', 'df', 'free', 'uptime', 'date', 'cal',
    'passwd', 'su', 'adduser', 'useradd', 'deluser', 'userdel', 'groupadd',
    'groups', 'hostname', 'uname', 'lsblk', 'fdisk', 'parted', 'dd',
    'xargs', 'awk', 'sed', 'sort', 'uniq', 'cut', 'tr', 'tee', 'less', 'more'
];


let cachedCurrentDir: string | null = null;
let cachedHomeDir: string | null = null;

/**
 * Refresh cached current directory after `cd` command
 */
export async function refreshCurrentDir(): Promise<string> {
    try {
        cachedCurrentDir = await invoke<string>('get_current_dir'); 
        return cachedCurrentDir;
    } catch (error) {
        return cachedCurrentDir || '';
    }
}


/**
 * Get the user's current directory
 */
async function getCurrentDirectory(): Promise<string> {
    if (cachedCurrentDir) return cachedCurrentDir
    return await refreshCurrentDir(); 
}

/**
 * Get the user's home directory
 */
async function getHomeDirectory(): Promise<string> {
    if (cachedHomeDir) return cachedHomeDir;
    try {
        cachedHomeDir = await invoke<string>('get_home_dir');
        return cachedHomeDir;
    } catch {
        return ''; 
    }
}


/**
 * Expands paths with ~ to use the home directory
 */
async function expandPath(path: string): Promise<string> {
    if (path.startsWith('~')) {
        const home = await getHomeDirectory();
        return path.replace(/^~/, home);
    }
    return path;
}

/**
 * Parse the input to extract path components
 */
function parsePathInput(input: string): { dirToSearch: string, prefix: string, searchPattern: string } {
    if (input.startsWith('/')) {
        const lastSlashIndex = input.lastIndexOf('/');
        if (lastSlashIndex === 0) {
            return {
                dirToSearch: '/',
                prefix: '/',
                searchPattern: input.substring(1).toLowerCase()
            };
        } else {
            return {
                dirToSearch: input.substring(0, lastSlashIndex),
                prefix: input.substring(0, lastSlashIndex + 1),
                searchPattern: input.substring(lastSlashIndex + 1).toLowerCase()
            };
        }
    }
    else if (input.includes('/')) {
        const lastSlashIndex = input.lastIndexOf('/');
        return {
            dirToSearch: input.substring(0, lastSlashIndex) || '.',
            prefix: input.substring(0, lastSlashIndex + 1),
            searchPattern: input.substring(lastSlashIndex + 1).toLowerCase()
        };
    }
    else {
        return {
            dirToSearch: '.',
            prefix: '',
            searchPattern: input.toLowerCase()
        };
    }
}

export interface AutocompleteResult {
    suggestions: string[];
    replacement?: string;
    commonPrefix?: string;
}

/**
 * Find common prefix among a list of strings
 */
function findCommonPrefix(strings: string[]): string {
    if (strings.length === 0) return '';
    if (strings.length === 1) return strings[0];

    let prefix = strings[0];
    for (let i = 1; i < strings.length; i++) {
        let j = 0;
        while (
            j < prefix.length &&
            j < strings[i].length &&
            prefix[j].toLowerCase() === strings[i][j].toLowerCase()
        ) {
            j++;
        }
        prefix = prefix.substring(0, j);
        if (prefix === '') break;
    }

    return prefix;
}

/**
 * Get autocompletion suggestions for the current input
 */
export async function getAutocompleteSuggestions(input: string): Promise<AutocompleteResult> {
    if (!input.trim()) {
        return { suggestions: [] };
    }
    const currentDir = await getCurrentDirectory();
    const words = input.split(' ');
    const lastWord = words[words.length - 1];
    const isFirstWord = words.length === 1;
    const beforeLastWord = input.substring(0, input.length - lastWord.length);

    // Special case: command with space at the end - suggest files in current directory
    if (input.endsWith(' ')) {
        const files = await invoke<string[]>('list_directory_contents', { path: currentDir });
        return { suggestions: files };
    }

    if (isFirstWord) {
        const matchingCommands = COMMON_COMMANDS.filter(cmd =>
            cmd.toLowerCase().startsWith(lastWord.toLowerCase())
        );

        if (matchingCommands.length > 0) {
            if (matchingCommands.length === 1) {
                return {
                    suggestions: matchingCommands,
                    replacement: matchingCommands[0]
                };
            }

            const commonPrefix = findCommonPrefix(matchingCommands);
            if (commonPrefix.length > lastWord.length) {
                return {
                    suggestions: matchingCommands,
                    commonPrefix: commonPrefix
                };
            }

            return { suggestions: matchingCommands };
        }
    }

    // File path completion
    const { dirToSearch, prefix, searchPattern } = parsePathInput(lastWord);

    try {
        let dirPath = dirToSearch;
        if (dirPath === '.') {
            dirPath = await getCurrentDirectory();
        }
        if (dirPath.startsWith('~')) {
            dirPath = await expandPath(dirPath);
        }

        const files = await invoke<string[]>('list_directory_contents', { path: dirPath });

        const matchingFiles = files.filter(file => {
            const fileName = file.includes('/') ?
                file.substring(file.lastIndexOf('/') + 1) : file;

            const cleanName = fileName.endsWith('/') || fileName.endsWith('*') ?
                fileName.slice(0, -1) : fileName;

            return cleanName.toLowerCase().startsWith(searchPattern);
        });

        const completions = matchingFiles.map(file => prefix + file);

        if (completions.length === 1) {
            return {
                suggestions: completions,
                replacement: beforeLastWord + completions[0]
            };
        }

        if (completions.length > 1) {
            const commonPrefix = findCommonPrefix(completions);
            if (commonPrefix.length > prefix.length) {
                return {
                    suggestions: completions,
                    commonPrefix: beforeLastWord + commonPrefix
                };
            }

            return { suggestions: completions };
        }
    } catch (error) {
        console.error('Error completing file path:', error);
    }

    return { suggestions: [] };
}
