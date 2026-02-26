import React from 'react';
import MarkdownRenderer from './MarkdownRenderer';

interface OutputFormatterProps {
    content: string;
    isMarkdown?: boolean;
}

const OutputFormatter: React.FC<OutputFormatterProps> = ({ content, isMarkdown = false }) => {
    // Render markdown for AI responses
    if (isMarkdown) {
        return <MarkdownRenderer content={content} />;
    }

    if (content.includes("{DIR}") || content.includes("{FILE}") || content.includes("{LINK}")) {
        const lines = content.split('\n');
        return (
            <>
                {lines.map((line, i) => {
                    // Directory line
                    if (line.includes("{DIR}")) {
                        const cleanLine = line.replace(/\{DIR\}/g, "").replace(/\{\/DIR\}/g, "");

                        let displayName = cleanLine;

                        // For Unix-style ls output with permissions
                        if (cleanLine.match(/^d[-rwx]{9}/)) {
                            const parts = cleanLine.split(/\s+/);
                            if (parts.length >= 8) {
                                const namePart = parts.slice(8).join(" ");

                                // Add a trailing slash if it doesn't have one
                                if (!namePart.endsWith("/")) {
                                    displayName = cleanLine.replace(namePart, `${namePart}/`);
                                }
                            }
                        }

                        return (
                            <div key={i} className="text-blue-400 font-bold">
                                <span className="mr-2">📁</span>
                                {displayName}
                            </div>
                        );
                    }
                    // Symlink line
                    else if (line.includes("{LINK}")) {
                        const cleanLine = line.replace(/\{LINK\}/g, "").replace(/\{\/LINK\}/g, "");
                        return (
                            <div key={i} className="text-cyan-300">
                                <span className="mr-2">🔗</span>
                                {cleanLine}
                            </div>
                        );
                    }
                    // File line
                    else if (line.includes("{FILE}")) {
                        const cleanLine = line.replace(/\{FILE\}/g, "").replace(/\{\/FILE\}/g, "");

                        const isExecutable = cleanLine.match(/-[-r][-w]x/);
                        const isImage = /\.(jpg|jpeg|png|gif|bmp|svg|webp)$/i.test(cleanLine);
                        const isDocument = /\.(pdf|doc|docx|txt|md|json|xml|html|css|js|ts|jsx|tsx)$/i.test(cleanLine);
                        const isArchive = /\.(zip|tar|gz|rar|7z)$/i.test(cleanLine);

                        let icon = '📄';
                        let colorClass = 'text-gray-300';

                        if (isExecutable) {
                            icon = '⚙️';
                            colorClass = 'text-green-300';
                        } else if (isImage) {
                            icon = '🖼️';
                            colorClass = 'text-purple-300';
                        } else if (isDocument) {
                            icon = '📝';
                            colorClass = 'text-yellow-200';
                        } else if (isArchive) {
                            icon = '📦';
                            colorClass = 'text-orange-300';
                        }

                        return (
                            <div key={i} className={colorClass}>
                                <span className="mr-2">{icon}</span>
                                {cleanLine}
                            </div>
                        );
                    }
                    else {
                        return <div key={i}>{line}</div>;
                    }
                })}
            </>
        );
    }
    return <>{content}</>;
};

export default OutputFormatter;
