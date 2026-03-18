pub fn css() -> &'static str {
    r#"* {
            box-sizing: border-box;
        }
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1000px; 
            margin: 40px auto; 
            padding: 0 20px;
            background: #1a1a1a;
            color: #e0e0e0;
        }
        h1 {
            color: #fff;
            border-bottom: 3px solid #c69b6d;
            padding-bottom: 12px;
        }
        h2 {
            color: #c69b6d;
            margin-top: 32px;
            margin-bottom: 16px;
        }
        .form-container {
            background: #2a2a2a;
            padding: 24px;
            border-radius: 8px;
            margin: 24px 0;
            box-shadow: 0 4px 6px rgba(0,0,0,0.3);
        }
        select, button { 
            padding: 10px 16px; 
            margin: 8px 8px 8px 0;
            font-size: 14px;
            border-radius: 4px;
            border: 1px solid #444;
            background: #333;
            color: #e0e0e0;
            min-width: 200px;
        }
        select:focus, button:focus {
            outline: 2px solid #c69b6d;
            outline-offset: 2px;
        }
        button { 
            background: #c69b6d;
            color: #1a1a1a;
            font-weight: 600;
            cursor: pointer;
            border: none;
            min-width: auto;
        }
        button:hover {
            background: #d4a574;
        }
        button:disabled {
            background: #555;
            color: #888;
            cursor: not-allowed;
        }
        #talents-container {
        }
        .talent-entry { 
            border: 1px solid #444;
            padding: 16px;
            margin: 12px 0;
            border-radius: 6px;
            background: #2a2a2a;
            animation: slideIn 0.4s cubic-bezier(0.16, 1, 0.3, 1);
            transform-origin: top;
            will-change: transform, opacity;
        }
        @keyframes slideIn {
            from { 
                opacity: 0; 
                transform: translateY(-20px) scale(0.95);
            }
            to { 
                opacity: 1; 
                transform: translateY(0) scale(1);
            }
        }
        .talent-entry h3 {
            margin-top: 0;
            margin-bottom: 8px;
            color: #c69b6d;
            font-size: 18px;
        }
        .talent-string { 
            font-family: 'Courier New', monospace;
            background: #1a1a1a;
            padding: 10px;
            border-radius: 4px;
            overflow-x: auto;
            font-size: 12px;
            margin: 12px 0;
            word-break: break-all;
        }
        .talent-entry a {
            color: #6db3c6;
            text-decoration: none;
            font-weight: 500;
        }
        .talent-entry a:hover {
            text-decoration: underline;
        }
        .iframe-container {
            overflow: hidden;
            position: relative;
        }
        .iframe-container iframe {
            display: block;
            margin: 0 auto;
        }
        @media (max-width: 900px) {
            .iframe-container iframe {
                transform: scale(0.7) !important;
                transform-origin: top center !important;
            }
        }
        #results {
            min-height: 100px;
        }
        .loading {
            text-align: center;
            padding: 40px;
            color: #888;
            font-style: italic;
        }
        .error {
            color: #e06c75;
            background: #2a1a1a;
            padding: 16px;
            border-radius: 6px;
            border-left: 4px solid #e06c75;
            margin: 16px 0;
        }
        .spinner {
            margin: 40px auto;
            width: 48px;
            height: 48px;
            border: 5px solid #444;
            border-top-color: #c69b6d;
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }

        @keyframes spin {
            to { transform: rotate(360deg); }
        }"#
}

pub fn toggle_script() -> &'static str {
    r#"document.addEventListener('click', (e) => {
    if (e.target.matches('.toggle-iframe-btn')) {
        const btn = e.target;
        const container = btn.closest('.talent-entry').querySelector('.iframe-container');
        
        if (container.style.display === 'none' || container.style.display === '') {
            container.style.display = 'block';
            container.style.height = '0px';
            const iframe = container.querySelector('iframe');
            const targetHeight = iframe.offsetHeight + 'px';
            container.style.transition = 'height 0.3s ease';
            requestAnimationFrame(() => { container.style.height = targetHeight; });
            btn.textContent = 'Hide Talent Calculator';
        } else {
            const currentHeight = container.offsetHeight + 'px';
            container.style.height = currentHeight;
            requestAnimationFrame(() => {
                container.style.transition = 'height 0.3s ease';
                container.style.height = '0px';
            });
            setTimeout(() => { container.style.display = 'none'; }, 300);
            btn.textContent = 'Show Talent Calculator';
        }
    }
});"#
}
