use crate::config::{ClassSpecs, Settings};
use crate::style;
use crate::warcraftlogs::TalentDataWithRank;

pub fn render_talent_entry(data: &TalentDataWithRank) -> String {
    let talent_string = &data.data.talent_string;

    let cast_json = serde_json::to_string(&data.data.cast_events)
        .unwrap_or_else(|_| "[]".to_string())
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    format!(
        r#"<div class="talent-entry" id="talent-entry-{rank}">
            <h3># {rank} - {name}</h3>
            <div class="talent-string">{talent_string}</div>

            <a href="{log_url}" target="_blank" rel="noopener">View Log →</a>

            <div class="entry-buttons">
                <button class="btn-secondary toggle-iframe-btn">
                    Show Talent Calculator
                </button>
                <button class="btn-secondary toggle-timeline-btn" data-rank="{rank}">
                    Show Timeline
                </button>
            </div>

            <div class="iframe-container" style="display:none; margin-top:12px; overflow:hidden;">
                <iframe src="https://www.wowhead.com/talent-calc/embed/blizzard/{talent_string}"
                    width="100%" height="580"
                    style="border:1px solid #444; border-radius:6px; display:block; min-width:980px;"></iframe>
            </div>

            <div class="cast-timeline"
                 id="cast-timeline-{rank}"
                 style="display:none;"
                 data-duration="{fight_duration_ms}"
                 data-events="{cast_json}">
            </div>
        </div>"#,
        rank              = data.rank,
        name              = data.data.name,
        talent_string     = talent_string,
        log_url           = data.data.log_url,
        fight_duration_ms = data.data.fight_duration_ms,
        cast_json         = cast_json,
    )
}

pub fn home(config: &ClassSpecs) -> String {
    let settings = Settings::load();

    let class_options: String = config
        .classes
        .iter()
        .map(|(name, _)| {
            let display_name = name.replace('_', " ");
            format!(r#"<option value="{}">{}</option>"#, name, display_name)
        })
        .collect::<Vec<_>>()
        .join("\n                ");

    let encounter_options: String = settings
        .current_encounters()
        .iter()
        .map(|enc| format!(r#"<option value="{}">{}</option>"#, enc.id, enc.name))
        .collect::<Vec<_>>()
        .join("\n                ");

    let region_options: String = ClassSpecs::get_regions()
        .iter()
        .map(|reg| format!(r#"<option value="{}">{}</option>"#, reg.code, reg.name))
        .collect::<Vec<_>>()
        .join("\n                ");

    let mode_options: String = ClassSpecs::get_modes()
        .iter()
        .map(|mode| format!(r#"<option value="{}">{}</option>"#, mode.name, mode.name))
        .collect::<Vec<_>>()
        .join("\n                ");

    let specs_map: String = config
        .classes
        .iter()
        .map(|(class_name, class_data)| {
            let specs = class_data
                .specs
                .iter()
                .map(|s| format!(r#""{}""#, s))
                .collect::<Vec<_>>()
                .join(", ");
            format!(r#""{}": [{}]"#, class_name, specs)
        })
        .collect::<Vec<_>>()
        .join(",\n            ");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Talent Trends</title>

    <script>
    {toggle_script}
    </script>

    <script>
    {timeline_script}
    </script>

    <style>
    {css}
    </style>
</head>
<body>
    <h1>Talent Trends</h1>

    <div class="form-container">
        <form id="talent-form">
            <select name="region" id="region" required>
                {region_options}
            </select>
            <select name="mode" id="mode" required>
                <option value="">Select Mode</option>
                {mode_options}
            </select>
            <select name="encounter" id="encounter" required>
                <option value="">Select Boss</option>
                {encounter_options}
            </select>
            <select name="class" id="class" required>
                <option value="">Select Class</option>
                {class_options}
            </select>
            <select name="spec" id="spec" required>
                <option value="">Select Spec</option>
            </select>
            <br>
            <input type="hidden" name="metric" id="metric-input" value="dps">
            <div class="metric-group" role="group" aria-label="Metric">
                <button type="button" class="metric-btn active" data-metric="dps">Damage</button>
                <button type="button" class="metric-btn"        data-metric="hps">Healing</button>
            </div>
            <button type="submit" id="submit-btn" disabled>Get Talents</button>
        </form>
    </div>

    <div id="results"></div>

    <script>
        const THEMES = {{
            dps: {{ accent: '#e84040', hover: '#ff5555', text: '#fff' }},
            hps: {{ accent: '#01C8AA', hover: '#02dfc0', text: '#0f1a18' }},
        }};

        function setTheme(metric) {{
            const t    = THEMES[metric] || THEMES.dps;
            const root = document.documentElement;
            root.style.setProperty('--accent',       t.accent);
            root.style.setProperty('--accent-hover', t.hover);
            root.style.setProperty('--accent-text',  t.text);
        }}

        const specsData = {{
            {specs_map}
        }};

        const regionSelect    = document.getElementById('region');
        const modeSelect      = document.getElementById('mode');
        const encounterSelect = document.getElementById('encounter');
        const classSelect     = document.getElementById('class');
        const specSelect      = document.getElementById('spec');
        const submitBtn       = document.getElementById('submit-btn');
        const resultsDiv      = document.getElementById('results');
        const metricInput     = document.getElementById('metric-input');

        // Populate spec options for a given class, optionally restoring a saved value.
        function populateSpecs(className, restoreValue) {{
            const prevValue = restoreValue !== undefined ? restoreValue : specSelect.value;
            specSelect.innerHTML = '<option value="">Select Spec</option>';
            if (className && specsData[className]) {{
                specsData[className].forEach(spec => {{
                    const option = document.createElement('option');
                    option.value = spec;
                    option.textContent = spec;
                    specSelect.appendChild(option);
                }});
                specSelect.disabled = false;
                // Restore previously selected spec if it exists in the new list
                if (prevValue && specsData[className].includes(prevValue)) {{
                    specSelect.value = prevValue;
                }}
            }} else {{
                specSelect.disabled = true;
            }}
        }}

        // On page load: restore spec list for whatever class the browser remembered,
        // restore metric button state and theme from the hidden input the browser remembered.
        document.addEventListener('DOMContentLoaded', () => {{
            // Restore spec dropdown
            if (classSelect.value) {{
                populateSpecs(classSelect.value, specSelect.value);
            }}

            // Restore metric button active state + theme
            const savedMetric = metricInput.value || 'dps';
            document.querySelectorAll('.metric-btn').forEach(btn => {{
                btn.classList.toggle('active', btn.dataset.metric === savedMetric);
            }});
            setTheme(savedMetric);

            updateSubmitButton();
        }});

        // Metric toggle buttons
        document.querySelectorAll('.metric-btn').forEach(btn => {{
            btn.addEventListener('click', () => {{
                document.querySelectorAll('.metric-btn').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                metricInput.value = btn.dataset.metric;
                setTheme(btn.dataset.metric);
            }});
        }});

        regionSelect.addEventListener('change', updateSubmitButton);
        modeSelect.addEventListener('change', updateSubmitButton);
        encounterSelect.addEventListener('change', updateSubmitButton);

        classSelect.addEventListener('change', (e) => {{
            populateSpecs(e.target.value);
            updateSubmitButton();
        }});

        specSelect.addEventListener('change', updateSubmitButton);

        function updateSubmitButton() {{
            submitBtn.disabled = !(
                regionSelect.value &&
                modeSelect.value &&
                encounterSelect.value &&
                classSelect.value &&
                specSelect.value
            );
        }}

        document.getElementById('talent-form').addEventListener('submit', async (e) => {{
            e.preventDefault();
            const formData = new FormData(e.target);
            const params   = new URLSearchParams(formData);

            resultsDiv.innerHTML = '<h2>Top 10 Talents</h2><div id="talents-container"></div><div id="loading-spinner" class="spinner"></div>';
            submitBtn.disabled = true;

            const eventSource = new EventSource('/api/talents?' + params);
            let firstData = true;

            eventSource.onmessage = (event) => {{
                if (firstData) {{
                    const spinner = document.getElementById('loading-spinner');
                    if (spinner) spinner.remove();
                    firstData = false;
                }}
                document.getElementById('talents-container')
                    .insertAdjacentHTML('beforeend', event.data);
            }};

            eventSource.addEventListener('complete', () => {{
                eventSource.close();
                updateSubmitButton();
            }});

            eventSource.onerror = () => {{
                eventSource.close();
                if (firstData) {{
                    resultsDiv.innerHTML = '<div class="error">Connection error. Please try again.</div>';
                }}
                updateSubmitButton();
            }};
        }});
    </script>
</body>
</html>
"#,
        css             = style::css(),
        toggle_script   = style::toggle_script(),
        timeline_script = style::timeline_script(),
        region_options  = region_options,
        mode_options    = mode_options,
        encounter_options = encounter_options,
        class_options   = class_options,
        specs_map       = specs_map,
    )
}
