import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import 'highlight.js/styles/github-dark.css';

interface MarkdownRendererProps {
  content: string;
}

const cleanContent = (raw: string) => {
  return raw
    .replace(/^\s*•\s*$/gm, '')
    .replace(/^\s*\.\s*$/gm, '')
    .replace(/\n{3,}/g, '\n\n')
    .trim();
};

const MarkdownRenderer: React.FC<MarkdownRendererProps> = ({ content }) => {
  const cleaned = cleanContent(content);

  return (
    <div className="markdown-content text-sm leading-relaxed">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeHighlight]}
        components={{
          h1: ({ children }) => (
            <h1 className="text-xl font-bold mt-3 mb-2 text-green-400">
              {children}
            </h1>
          ),
          h2: ({ children }) => (
            <h2 className="text-lg font-bold mt-3 mb-1 text-green-400">
              {children}
            </h2>
          ),
          h3: ({ children }) => (
            <h3 className="text-base font-bold mt-2 mb-1 text-green-400">
              {children}
            </h3>
          ),
          code({ inline, className, children, ...props }: any) {
            if (inline) {
              return (
                <code
                  className="bg-gray-800 px-1 py-0.5 rounded text-sm"
                  {...props}
                >
                  {children}
                </code>
              );
            }
            // For code blocks.
            return <code className={className} {...props}>{children}</code>;
          },
          pre({ children }: any) {
            // Style the pre wrapper for code blocks
            return (
              <pre className="bg-gray-800 rounded px-3 py-2 overflow-x-auto my-2">
                {children}
              </pre>
            );
          },
          a: ({ href, children }) => (
            <a
              href={href}
              className="text-blue-400 hover:text-blue-300 underline"
              target="_blank"
              rel="noopener noreferrer"
            >
              {children}
            </a>
          ),
          blockquote: ({ children }) => (
            <blockquote className="border-l-4 border-green-500 pl-3 my-2 italic text-gray-400">
              {children}
            </blockquote>
          ),
          strong: ({ children }) => (
            <strong className="font-semibold text-green-300">
              {children}
            </strong>
          ),
          em: ({ children }) => (
            <em className="italic text-gray-300">
              {children}
            </em>
          ),
        }}
      >
        {cleaned}
      </ReactMarkdown>
    </div>
  );
};

export default MarkdownRenderer;
