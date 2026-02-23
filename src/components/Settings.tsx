import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface AiSettings {
    providers: {
        [key: string]: ProviderConfig;
    };
}

interface ProviderConfig {
    api_key: string;
    model: string;
    is_default: boolean;
}

interface ModelInfo {
    id: string;
    name: string;
}

interface SettingsProps {
    isOpen: boolean;
    onClose: () => void;
}

const PROVIDERS = [
    { id: 'openai', name: 'OpenAI' },
    { id: 'anthropic', name: 'Anthropic (Claude)' },
    { id: 'gemini', name: 'Google Gemini' },
    { id: 'groq', name: 'Groq' },
    { id: 'openrouter', name: 'OpenRouter' }
];

const Settings: React.FC<SettingsProps> = ({ isOpen, onClose }) => {
    const [activeTab, setActiveTab] = useState<'ai' | 'about'>('ai');
    const [provider, setProvider] = useState('openai');
    const [apiKey, setApiKey] = useState('');
    const [model, setModel] = useState('');
    const [models, setModels] = useState<ModelInfo[]>([]);
    const [modelsCache, setModelsCache] = useState<{ [key: string]: ModelInfo[] }>({});
    const [isValidating, setIsValidating] = useState(false);
    const [isLoadingModels, setIsLoadingModels] = useState(false);
    const [isSaving, setIsSaving] = useState(false);
    const [validationStatus, setValidationStatus] = useState<'idle' | 'valid' | 'invalid'>('idle');
    const [errorMessage, setErrorMessage] = useState('');
    const [successMessage, setSuccessMessage] = useState('');
    const [allSettings, setAllSettings] = useState<AiSettings>({ providers: {} });

    useEffect(() => {
        if (isOpen) {
            loadSettings();
        }
    }, [isOpen]);

    useEffect(() => {
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && isOpen) {
                onClose();
            }
        };

        document.addEventListener('keydown', handleEscape);
        return () => document.removeEventListener('keydown', handleEscape);
    }, [isOpen, onClose]);

    const loadSettings = async () => {
        try {
            const settings = await invoke<AiSettings>('get_ai_settings');
            setAllSettings(settings);
            
            // Load the current provider's settings if available
            if (settings.providers[provider]) {
                const providerConfig = settings.providers[provider];
                setApiKey(providerConfig.api_key);
                setModel(providerConfig.model);
                
                if (providerConfig.api_key) {
                    setValidationStatus('valid');
                    // Only fetch models if not already cached
                    if (!modelsCache[provider]) {
                        await fetchModels(provider, providerConfig.api_key);
                    } else {
                        setModels(modelsCache[provider]);
                    }
                }
            }
        } catch (error) {
            console.error('Failed to load settings:', error);
        }
    };

    const handleProviderChange = (newProvider: string) => {
        setProvider(newProvider);
        setErrorMessage('');
        setSuccessMessage('');
        
        // Load settings for the new provider if available
        if (allSettings.providers[newProvider]) {
            const providerConfig = allSettings.providers[newProvider];
            setApiKey(providerConfig.api_key);
            setModel(providerConfig.model);
            setValidationStatus('valid');
            
            // Use cached models if available, otherwise fetch
            if (modelsCache[newProvider]) {
                setModels(modelsCache[newProvider]);
            } else {
                fetchModels(newProvider, providerConfig.api_key);
            }
        } else {
            // Reset fields for new provider
            setApiKey('');
            setModel('');
            setValidationStatus('idle');
            setModels([]);
        }
    };

    const validateApiKey = async () => {
        if (!apiKey.trim()) {
            setErrorMessage('Please enter an API key');
            return;
        }

        setIsValidating(true);
        setErrorMessage('');
        setSuccessMessage('');

        try {
            const isValid = await invoke<boolean>('validate_provider_api_key', {
                provider,
                apiKey
            });

            if (isValid) {
                setValidationStatus('valid');
                setSuccessMessage('API key validated successfully!');
                // Only fetch models if not already cached for this provider
                if (!modelsCache[provider]) {
                    await fetchModels(provider, apiKey);
                } else {
                    setModels(modelsCache[provider]);
                }
            } else {
                setValidationStatus('invalid');
                setErrorMessage('Invalid API key');
            }
        } catch (error: any) {
            setValidationStatus('invalid');
            setErrorMessage(error.toString());
        } finally {
            setIsValidating(false);
        }
    };

    const fetchModels = async (prov: string, key: string) => {
        setIsLoadingModels(true);
        try {
            const fetchedModels = await invoke<ModelInfo[]>('fetch_available_models', {
                provider: prov,
                apiKey: key
            });
            setModels(fetchedModels);
            
            // Cache the models for this provider
            setModelsCache(prev => ({
                ...prev,
                [prov]: fetchedModels
            }));
            
            if (fetchedModels.length > 0 && !model) {
                setModel(fetchedModels[0].id);
            }
        } catch (error: any) {
            setErrorMessage(`Failed to fetch models: ${error}`);
        } finally {
            setIsLoadingModels(false);
        }
    };

    const handleSave = async () => {
        if (validationStatus !== 'valid') {
            setErrorMessage('Please validate your API key first');
            return;
        }

        if (!model) {
            setErrorMessage('Please select a model');
            return;
        }

        setIsSaving(true);
        setErrorMessage('');
        setSuccessMessage('');

        try {
            // Always set the current provider as default when saving
            await invoke('save_ai_settings', {
                provider,
                apiKey,
                model,
                isDefault: true
            });
            
            // Update local settings
            setAllSettings(prev => ({
                providers: {
                    ...prev.providers,
                    [provider]: {
                        api_key: apiKey,
                        model,
                        is_default: true
                    }
                }
            }));
            
            setSuccessMessage('Settings saved successfully! This provider is now your default.');
            setTimeout(() => {
                onClose();
            }, 1000);
        } catch (error: any) {
            setErrorMessage(`Failed to save settings: ${error}`);
        } finally {
            setIsSaving(false);
        }
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div className="bg-gray-800 rounded-lg w-full max-w-3xl shadow-xl flex overflow-hidden relative" style={{ height: '600px' }}>
                {/* Close Button - Top Right Corner */}
                <button
                    onClick={onClose}
                    className="absolute top-4 right-4 z-10 text-gray-300 hover:text-white transition-colors text-3xl font-bold leading-none w-8 h-8 flex items-center justify-center"
                    title="Close (ESC)"
                >
                    ×
                </button>

                {/* Left Sidebar with Tabs */}
                <div className="w-64 bg-gray-900 p-4 flex flex-col">
                    <h2 className="text-xl font-bold text-white mb-6">Settings</h2>

                    <nav className="space-y-2">
                        <button
                            onClick={() => setActiveTab('ai')}
                            className={`w-full text-left px-4 py-3 rounded transition-colors ${
                                activeTab === 'ai'
                                    ? 'bg-gray-700 text-white'
                                    : 'text-gray-400 hover:bg-gray-800 hover:text-white'
                            }`}
                        >
                            AI Settings
                        </button>
                        <button
                            onClick={() => setActiveTab('about')}
                            className={`w-full text-left px-4 py-3 rounded transition-colors ${
                                activeTab === 'about'
                                    ? 'bg-gray-700 text-white'
                                    : 'text-gray-400 hover:bg-gray-800 hover:text-white'
                            }`}
                        >
                            About
                        </button>
                    </nav>
                </div>

                {/* Right Content Area */}
                <div className="flex-1 p-6 overflow-y-auto">
                    {activeTab === 'ai' && (
                        <div className="space-y-4">
                            <h3 className="text-2xl font-bold text-white mb-4">AI Configuration</h3>

                            {/* Provider Selection */}
                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    AI Provider
                                </label>
                                <select
                                    value={provider}
                                    onChange={(e) => handleProviderChange(e.target.value)}
                                    className="w-full bg-gray-700 text-white rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                >
                                    {PROVIDERS.map((p) => {
                                        const isDefault = allSettings.providers[p.id]?.is_default;
                                        return (
                                            <option key={p.id} value={p.id}>
                                                {p.name}{isDefault ? ' (Default)' : ''}
                                            </option>
                                        );
                                    })}
                                </select>
                            </div>

                            {/* API Key Input */}
                            <div>
                                <label className="block text-sm font-medium text-gray-300 mb-2">
                                    API Key
                                </label>
                                <div className="flex gap-2">
                                    <input
                                        type="password"
                                        value={apiKey}
                                        onChange={(e) => {
                                            setApiKey(e.target.value);
                                            setValidationStatus('idle');
                                            setErrorMessage('');
                                            setSuccessMessage('');
                                        }}
                                        placeholder="Enter your API key"
                                        className="flex-1 bg-gray-700 text-white rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    />
                                    <button
                                        onClick={validateApiKey}
                                        disabled={isValidating || !apiKey.trim()}
                                        className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white px-4 py-2 rounded transition-colors"
                                    >
                                        {isValidating ? 'Validating...' : 'Validate'}
                                    </button>
                                </div>
                            </div>

                            {/* Model Selection */}
                            {validationStatus === 'valid' && (
                                <div>
                                    <label className="block text-sm font-medium text-gray-300 mb-2">
                                        Model
                                    </label>
                                    {isLoadingModels ? (
                                        <div className="text-gray-400 text-sm">Loading models...</div>
                                    ) : models.length > 0 ? (
                                        <select
                                            value={model}
                                            onChange={(e) => setModel(e.target.value)}
                                            className="w-full bg-gray-700 text-white rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        >
                                            {models.map((m) => (
                                                <option key={m.id} value={m.id}>
                                                    {m.name}
                                                </option>
                                            ))}
                                        </select>
                                    ) : (
                                        <div className="text-gray-400 text-sm">No models available</div>
                                    )}
                                </div>
                            )}

                            {/* Error Message */}
                            {errorMessage && (
                                <div className="bg-red-900 bg-opacity-50 text-red-200 px-3 py-2 rounded text-sm">
                                    {errorMessage}
                                </div>
                            )}

                            {/* Success Message */}
                            {successMessage && (
                                <div className="bg-green-900 bg-opacity-50 text-green-200 px-3 py-2 rounded text-sm">
                                    {successMessage}
                                </div>
                            )}

                            {/* Action Buttons */}
                            <div className="flex gap-2 pt-4">
                                <button
                                    onClick={handleSave}
                                    disabled={isSaving || validationStatus !== 'valid' || !model}
                                    className="flex-1 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 text-white px-4 py-2 rounded transition-colors"
                                >
                                    {isSaving ? 'Saving...' : 'Save Settings'}
                                </button>
                                <button
                                    onClick={onClose}
                                    className="flex-1 bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded transition-colors"
                                >
                                    Cancel
                                </button>
                            </div>
                        </div>
                    )}

                    {activeTab === 'about' && (
                        <div className="space-y-4">
                            <h3 className="text-2xl font-bold text-white mb-4">About Term</h3>
                            
                            <div className="space-y-4 text-gray-300">
                                <div>
                                    <h4 className="text-lg font-semibold text-white mb-2">Version</h4>
                                    <p>Term v0.1.0</p>
                                    <p>Created by <a href="https://github.com/sapatevaibhav">sapatevaibhav</a></p>
                                </div>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export default Settings;
