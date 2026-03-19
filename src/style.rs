pub fn css() -> &'static str {
    r#"* {
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
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
        button:hover { background: #d4a574; }
        button:disabled {
            background: #555;
            color: #888;
            cursor: not-allowed;
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
            from { opacity: 0; transform: translateY(-20px) scale(0.95); }
            to   { opacity: 1; transform: translateY(0) scale(1); }
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
        .talent-entry a:hover { text-decoration: underline; }
        .entry-buttons {
            display: flex;
            gap: 8px;
            flex-wrap: wrap;
            margin-top: 12px;
        }
        .btn-secondary {
            background: #3a3a3a;
            color: #c0c0c0;
            border: 1px solid #555;
            font-weight: 500;
        }
        .btn-secondary:hover { background: #444; color: #e0e0e0; }
        .iframe-container {
            overflow: hidden;
            position: relative;
        }
        .iframe-container iframe { display: block; margin: 0 auto; }
        @media (max-width: 900px) {
            .iframe-container iframe {
                transform: scale(0.7) !important;
                transform-origin: top center !important;
            }
        }
        #results { min-height: 100px; }
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
        @keyframes spin { to { transform: rotate(360deg); } }

        /* ── Cast timeline ── */
        .cast-timeline {
            margin-top: 12px;
            border: 1px solid #383838;
            border-radius: 6px;
            overflow: hidden;
        }
        .ct-toolbar {
            display: flex;
            align-items: center;
            gap: 16px;
            padding: 8px 12px;
            background: #222;
            border-bottom: 1px solid #333;
            flex-wrap: wrap;
        }
        .ct-toolbar label {
            display: flex;
            align-items: center;
            gap: 5px;
            font-size: 12px;
            color: #888;
            cursor: pointer;
            user-select: none;
        }
        .ct-toolbar input[type=range] { width: 120px; accent-color: #c69b6d; }
        .ct-toolbar span { font-size: 12px; color: #666; }
        .ct-scroll {
            overflow-x: auto;
            background: #181818;
            padding: 10px 12px 12px;
        }
        .ct-inner { min-width: 600px; }
        .ct-axis-row {
            display: flex;
            align-items: flex-end;
            margin-bottom: 4px;
        }
        .ct-label-col {
            width: 210px;
            min-width: 210px;
            flex-shrink: 0;
        }
        .ct-axis {
            flex: 1;
            position: relative;
            height: 18px;
            border-bottom: 1px solid #2e2e2e;
        }
        .ct-tick {
            position: absolute;
            font-size: 10px;
            color: #444;
            font-family: 'Courier New', monospace;
            transform: translateX(-50%);
            bottom: 2px;
            white-space: nowrap;
        }
        .ct-row {
            display: flex;
            align-items: center;
            height: 26px;
            margin: 1px 0;
        }
        .ct-row:hover .ct-bar { background: #222; }
        .ct-label {
            width: 210px;
            min-width: 210px;
            flex-shrink: 0;
            display: flex;
            align-items: center;
            gap: 6px;
            padding-right: 10px;
            overflow: hidden;
        }
        .ct-icon {
            width: 18px;
            height: 18px;
            border-radius: 3px;
            flex-shrink: 0;
            image-rendering: pixelated;
        }
        .ct-name {
            font-size: 11px;
            color: #a0a0a0;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            flex: 1;
        }
        .ct-count {
            font-size: 10px;
            color: #555;
            flex-shrink: 0;
            font-family: 'Courier New', monospace;
        }
        .ct-bar {
            flex: 1;
            position: relative;
            height: 18px;
            background: #1e1e1e;
            border-radius: 2px;
            transition: background 0.1s;
        }
        .ct-mark {
            position: absolute;
            width: 3px;
            height: 14px;
            background: #c69b6d;
            border-radius: 1px;
            top: 2px;
            transform: translateX(-50%);
            opacity: 0.85;
            cursor: default;
        }
        .ct-mark:hover {
            background: #e8b87a;
            opacity: 1;
            z-index: 2;
            width: 4px;
        }
        .ct-empty {
            padding: 16px;
            color: #555;
            font-size: 12px;
            text-align: center;
        }"#
}

pub fn toggle_script() -> &'static str {
    // The toggle handler now also lazily initialises the timeline on first open.
    // Data lives in data-events / data-duration attributes on the container div,
    // so no inline <script> is needed in the SSE-streamed HTML fragments
    // (injected <script> tags don't execute via insertAdjacentHTML).
    r#"document.addEventListener('click', (e) => {

    // ── Talent calculator iframe toggle ───────────────────────────────────────
    if (e.target.matches('.toggle-iframe-btn')) {
        const btn       = e.target;
        const entry     = btn.closest('.talent-entry');
        const container = entry.querySelector('.iframe-container');

        if (container.style.display === 'none' || container.style.display === '') {
            container.style.display  = 'block';
            container.style.height   = '0px';
            const iframe             = container.querySelector('iframe');
            const targetHeight       = iframe.offsetHeight + 'px';
            container.style.transition = 'height 0.3s ease';
            requestAnimationFrame(() => { container.style.height = targetHeight; });
            btn.textContent = 'Hide Talent Calculator';
        } else {
            const currentHeight        = container.offsetHeight + 'px';
            container.style.height     = currentHeight;
            requestAnimationFrame(() => {
                container.style.transition = 'height 0.3s ease';
                container.style.height     = '0px';
            });
            setTimeout(() => { container.style.display = 'none'; }, 300);
            btn.textContent = 'Show Talent Calculator';
        }
    }

    // ── Cast timeline toggle ──────────────────────────────────────────────────
    if (e.target.matches('.toggle-timeline-btn')) {
        const btn       = e.target;
        const rank      = btn.dataset.rank;
        const container = document.getElementById('cast-timeline-' + rank);
        if (!container) return;

        const showing = container.style.display === 'block';

        if (!showing) {
            container.style.display = 'block';
            btn.textContent = 'Hide Timeline';

            // Lazy init: render on first open only (guard with a flag attribute)
            if (!container.dataset.rendered) {
                container.dataset.rendered = '1';
                try {
                    const events   = JSON.parse(container.dataset.events || '[]');
                    const duration = parseInt(container.dataset.duration || '0', 10);
                    renderTimeline('cast-timeline-' + rank, events, duration);
                } catch (err) {
                    container.innerHTML =
                        '<div class="ct-empty">Failed to parse timeline data: ' + err.message + '</div>';
                }
            }
        } else {
            container.style.display = 'none';
            btn.textContent = 'Show Timeline';
        }
    }

});"#
}

pub fn timeline_script() -> &'static str {
    r#"function renderTimeline(id, events, durationMs) {
    const el = document.getElementById(id);
    if (!el) return;

    if (!events || !events.length || !durationMs) {
        el.innerHTML = '<div class="ct-empty">No cast data available.</div>';
        return;
    }

    // Group by ability ID
    const byId = {};
    for (const ev of events) {
        if (!byId[ev.id]) byId[ev.id] = { name: ev.name, icon: ev.icon, times: [] };
        byId[ev.id].times.push(ev.t);
    }

    // Sort rows by first cast time ascending
    const allRows = Object.values(byId).sort((a, b) => a.times[0] - b.times[0]);

    function esc(s) {
        return String(s).replace(/[<>&"]/g, c =>
            ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', '"': '&quot;' }[c]));
    }

    function fmt(ms) {
        const s = Math.floor(ms / 1000);
        return Math.floor(s / 60) + ':' + String(s % 60).padStart(2, '0');
    }

    function build(maxCasts) {
        const rows = allRows.filter(a => a.times.length <= maxCasts);
        if (!rows.length) {
            return '<div class="ct-empty">No abilities match this filter.</div>';
        }

        const durSec   = durationMs / 1000;
        const interval = durSec > 300 ? 60 : 30;

        let ticks = '';
        for (let s = 0; s <= Math.ceil(durSec); s += interval) {
            const pct = (s / durSec * 100).toFixed(2);
            ticks += `<span class="ct-tick" style="left:${pct}%">${Math.floor(s/60)}:${String(s%60).padStart(2,'0')}</span>`;
        }

        let rowsHtml = '';
        for (const a of rows) {
            const iconUrl = 'https://assets.rpglogs.com/img/warcraft/abilities/' + a.icon;
            const marks   = a.times.map(t => {
                const pct = (t / durationMs * 100).toFixed(2);
                return `<div class="ct-mark" style="left:${pct}%" title="${esc(a.name)} @ ${fmt(t)}"></div>`;
            }).join('');

            rowsHtml += `
            <div class="ct-row">
                <div class="ct-label">
                    <img class="ct-icon" src="${iconUrl}" alt=""
                         onerror="this.style.display='none'">
                    <span class="ct-name">${esc(a.name)}</span>
                    <span class="ct-count">x${a.times.length}</span>
                </div>
                <div class="ct-bar">${marks}</div>
            </div>`;
        }

        return `<div class="ct-inner">
            <div class="ct-axis-row">
                <div class="ct-label-col"></div>
                <div class="ct-axis">${ticks}</div>
            </div>
            ${rowsHtml}
        </div>`;
    }

    const defaultMax = 20;
    const uid        = id; // stable closure key

    window._ctBuilders = window._ctBuilders || {};
    window._ctBuilders[uid] = build;

    el.innerHTML = `
        <div class="ct-toolbar">
            <label>
                Max casts per ability
                <input type="range" min="1" max="50" value="${defaultMax}"
                    oninput="
                        document.getElementById('ct-val-${uid}').textContent = this.value;
                        document.getElementById('ct-body-${uid}').innerHTML =
                            window._ctBuilders['${uid}'](+this.value);
                    ">
            </label>
            <span id="ct-val-${uid}">${defaultMax}</span>
            <span style="margin-left:auto;color:#444">${fmt(durationMs)} fight</span>
        </div>
        <div class="ct-scroll">
            <div id="ct-body-${uid}"></div>
        </div>`;

    document.getElementById('ct-body-' + uid).innerHTML = build(defaultMax);
}"#
}
