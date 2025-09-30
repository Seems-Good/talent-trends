use crate::config::ClassSpecs;
use crate::warcraftlogs::TalentDataWithRank;

pub fn render_talent_entry(data: &TalentDataWithRank) -> String {
    format!(
        r#"<div class="talent-entry">
            <h3>#{} - {}</h3>
            <div class="talent-string">{}</div>
            <a href="{}" target="_blank" rel="noopener">View Log →</a>
        </div>"#,
        data.rank, data.data.name, data.data.talent_string, data.data.log_url
    )
}

pub fn home(config: &ClassSpecs) -> String {
    let class_options: String = config.classes
        .iter()
        .map(|(name, _)| {
            let display_name = name.replace('_', " ");
            format!(r#"<option value="{}">{}</option>"#, name, display_name)
        })
        .collect::<Vec<_>>()
        .join("\n                ");
    
    let encounter_options: String = crate::config::get_encounters()
        .iter()
        .map(|enc| {
            format!(r#"<option value="{}">{}</option>"#, enc.id, enc.name)
        })
        .collect::<Vec<_>>()
        .join("\n                ");
    
    let region_options: String = crate::config::get_regions()
        .iter()
        .map(|reg| {
            format!(r#"<option value="{}">{}</option>"#, reg.code, reg.name)
        })
        .collect::<Vec<_>>()
        .join("\n                ");
    
    let specs_map: String = config.classes
        .iter()
        .map(|(class_name, class_data)| {
            let specs = class_data.specs
                .iter()
                .map(|s| format!(r#""{}""#, s))
                .collect::<Vec<_>>()
                .join(", ");
            format!(r#""{}": [{}]"#, class_name, specs)
        })
        .collect::<Vec<_>>()
        .join(",\n            ");

    format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Talent Trends - WarcraftLogs</title>
    <style>
        * {{
            box-sizing: border-box;
        }}
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 900px; 
            margin: 40px auto; 
            padding: 0 20px;
            background: #1a1a1a;
            color: #e0e0e0;
        }}
        h1 {{
            color: #fff;
            border-bottom: 3px solid #c69b6d;
            padding-bottom: 12px;
        }}
        h2 {{
            color: #c69b6d;
            margin-top: 32px;
            margin-bottom: 16px;
        }}
        .form-container {{
            background: #2a2a2a;
            padding: 24px;
            border-radius: 8px;
            margin: 24px 0;
            box-shadow: 0 4px 6px rgba(0,0,0,0.3);
        }}
        select, button {{ 
            padding: 10px 16px; 
            margin: 8px 8px 8px 0;
            font-size: 14px;
            border-radius: 4px;
            border: 1px solid #444;
            background: #333;
            color: #e0e0e0;
            min-width: 200px;
        }}
        select:focus, button:focus {{
            outline: 2px solid #c69b6d;
            outline-offset: 2px;
        }}
        button {{ 
            background: #c69b6d;
            color: #1a1a1a;
            font-weight: 600;
            cursor: pointer;
            border: none;
            min-width: auto;
        }}
        button:hover {{
            background: #d4a574;
        }}
        button:disabled {{
            background: #555;
            color: #888;
            cursor: not-allowed;
        }}
        #talents-container {{
            /* Container for smooth additions */
        }}
        .talent-entry {{ 
            border: 1px solid #444;
            padding: 16px;
            margin: 12px 0;
            border-radius: 6px;
            background: #2a2a2a;
            animation: slideIn 0.4s cubic-bezier(0.16, 1, 0.3, 1);
            transform-origin: top;
            will-change: transform, opacity;
        }}
        @keyframes slideIn {{
            from {{ 
                opacity: 0; 
                transform: translateY(-20px) scale(0.95);
            }}
            to {{ 
                opacity: 1; 
                transform: translateY(0) scale(1);
            }}
        }}
        .talent-entry h3 {{
            margin-top: 0;
            margin-bottom: 8px;
            color: #c69b6d;
            font-size: 18px;
        }}
        .talent-string {{ 
            font-family: 'Courier New', monospace;
            background: #1a1a1a;
            padding: 10px;
            border-radius: 4px;
            overflow-x: auto;
            font-size: 12px;
            margin: 12px 0;
            word-break: break-all;
        }}
        .talent-entry a {{
            color: #6db3c6;
            text-decoration: none;
            font-weight: 500;
        }}
        .talent-entry a:hover {{
            text-decoration: underline;
        }}
        #results {{
            min-height: 100px;
        }}
        .loading {{
            text-align: center;
            padding: 40px;
            color: #888;
            font-style: italic;
        }}
        .error {{
            color: #e06c75;
            background: #2a1a1a;
            padding: 16px;
            border-radius: 6px;
            border-left: 4px solid #e06c75;
            margin: 16px 0;
        }}
    </style>
</head>
<body>
    <h1>⚔️ WarcraftLogs Talent Trends</h1>
    
    <div class="form-container">
        <form id="talent-form">
            <select name="region" id="region" required>
                {}
            </select>
            <select name="encounter" id="encounter" required>
                <option value="">Select Boss</option>
                {}
            </select>
            <select name="class" id="class" required>
                <option value="">Select Class</option>
                {}
            </select>
            <select name="spec" id="spec" required disabled>
                <option value="">Select Spec</option>
            </select>
            <button type="submit" id="submit-btn" disabled>Get Talents</button>
        </form>
    </div>
    
    <div id="results"></div>
    
    <script>
        const specsData = {{
            {}
        }};
        
        const regionSelect = document.getElementById('region');
        const encounterSelect = document.getElementById('encounter');
        const classSelect = document.getElementById('class');
        const specSelect = document.getElementById('spec');
        const submitBtn = document.getElementById('submit-btn');
        const resultsDiv = document.getElementById('results');
        
        regionSelect.addEventListener('change', updateSubmitButton);
        encounterSelect.addEventListener('change', updateSubmitButton);
        
        classSelect.addEventListener('change', (e) => {{
            const selectedClass = e.target.value;
            specSelect.innerHTML = '<option value="">Select Spec</option>';
            
            if (selectedClass && specsData[selectedClass]) {{
                specsData[selectedClass].forEach(spec => {{
                    const option = document.createElement('option');
                    option.value = spec;
                    option.textContent = spec;
                    specSelect.appendChild(option);
                }});
                specSelect.disabled = false;
            }} else {{
                specSelect.disabled = true;
            }}
            updateSubmitButton();
        }});
        
        specSelect.addEventListener('change', updateSubmitButton);
        
        function updateSubmitButton() {{
            const allSelected = regionSelect.value && encounterSelect.value && classSelect.value && specSelect.value;
            submitBtn.disabled = !allSelected;
        }}
        
        document.getElementById('talent-form').addEventListener('submit', async (e) => {{
            e.preventDefault();
            const formData = new FormData(e.target);
            const params = new URLSearchParams(formData);
            
            resultsDiv.innerHTML = '<h2>Top 10 Talents</h2><div class="loading">Loading talent strings...</div>';
            submitBtn.disabled = true;
            
            const eventSource = new EventSource(`/api/talents?${{params}}`);
            let firstData = true;
            
            eventSource.onmessage = (event) => {{
                if (firstData) {{
                    resultsDiv.innerHTML = '<h2>Top 10 Talents</h2><div id="talents-container"></div>';
                    firstData = false;
                }}
                const container = document.getElementById('talents-container');
                container.insertAdjacentHTML('beforeend', event.data);
            }};
            
            eventSource.addEventListener('complete', () => {{
                eventSource.close();
                updateSubmitButton();
            }});
            
            eventSource.onerror = (err) => {{
                console.error('EventSource error:', err);
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
"#, region_options, encounter_options, class_options, specs_map)
}




// use crate::config::ClassSpecs;
// //use crate::warcraftlogs::TalentData;
// use crate::warcraftlogs::TalentDataWithRank;
//
// pub fn render_talent_entry(data: &TalentDataWithRank) -> String {
//     format!(
//         r#"<div class="talent-entry">
//             <h3>#{} - {}</h3>
//             <div class="talent-string">{}</div>
//             <a href="{}" target="_blank" rel="noopener">View Log →</a>
//         </div>"#,
//         data.rank, data.data.name, data.data.talent_string, data.data.log_url
//     )
// }
//
// pub fn home(config: &ClassSpecs) -> String {
//     let class_options: String = config.classes
//         .iter()
//         .map(|(name, _)| {
//             let display_name = name.replace('_', " ");
//             format!(r#"<option value="{}">{}</option>"#, name, display_name)
//         })
//         .collect::<Vec<_>>()
//         .join("\n                ");
//
//     let encounter_options: String = crate::config::get_encounters()
//         .iter()
//         .map(|enc| {
//             format!(r#"<option value="{}">{}</option>"#, enc.id, enc.name)
//         })
//         .collect::<Vec<_>>()
//         .join("\n                ");
//
//     let region_options: String = crate::config::get_regions()
//         .iter()
//         .map(|reg| {
//             format!(r#"<option value="{}">{}</option>"#, reg.code, reg.name)
//         })
//         .collect::<Vec<_>>()
//         .join("\n                ");
//
//     let specs_map: String = config.classes
//         .iter()
//         .map(|(class_name, class_data)| {
//             let specs = class_data.specs
//                 .iter()
//                 .map(|s| format!(r#""{}""#, s))
//                 .collect::<Vec<_>>()
//                 .join(", ");
//             format!(r#""{}": [{}]"#, class_name, specs)
//         })
//         .collect::<Vec<_>>()
//         .join(",\n            ");
//
//     format!(r#"
// <!DOCTYPE html>
// <html lang="en">
// <head>
//     <meta charset="UTF-8">
//     <meta name="viewport" content="width=device-width, initial-scale=1.0">
//     <title>Talent Trends</title>
//     <style>
//         body {{ 
//             font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
//             max-width: 900px; 
//             margin: 40px auto; 
//             padding: 0 20px;
//             background: #1a1a1a;
//             color: #e0e0e0;
//         }}
//         h1 {{
//             color: #fff;
//             border-bottom: 3px solid #c69b6d;
//             padding-bottom: 12px;
//         }}
//         h2 {{
//             color: #c69b6d;
//             margin-top: 32px;
//         }}
//         .form-container {{
//             background: #2a2a2a;
//             padding: 24px;
//             border-radius: 8px;
//             margin: 24px 0;
//             box-shadow: 0 4px 6px rgba(0,0,0,0.3);
//         }}
//         select, button {{ 
//             padding: 10px 16px; 
//             margin: 8px 8px 8px 0;
//             font-size: 14px;
//             border-radius: 4px;
//             border: 1px solid #444;
//             background: #333;
//             color: #e0e0e0;
//             min-width: 200px;
//         }}
//         select:focus, button:focus {{
//             outline: 2px solid #c69b6d;
//             outline-offset: 2px;
//         }}
//         button {{ 
//             background: #c69b6d;
//             color: #1a1a1a;
//             font-weight: 600;
//             cursor: pointer;
//             border: none;
//             min-width: auto;
//         }}
//         button:hover {{
//             background: #d4a574;
//         }}
//         button:disabled {{
//             background: #555;
//             color: #888;
//             cursor: not-allowed;
//         }}
//         .talent-entry {{ 
//             border: 1px solid #444;
//             padding: 16px;
//             margin: 12px 0;
//             border-radius: 6px;
//             background: #2a2a2a;
//             animation: fadeIn 0.3s ease-in;
//         }}
//         @keyframes fadeIn {{
//             from {{ opacity: 0; transform: translateY(-10px); }}
//             to {{ opacity: 1; transform: translateY(0); }}
//         }}
//         .talent-entry h3 {{
//             margin-top: 0;
//             color: #c69b6d;
//         }}
//         .talent-string {{ 
//             font-family: 'Courier New', monospace;
//             background: #1a1a1a;
//             padding: 8px;
//             border-radius: 4px;
//             overflow-x: auto;
//             font-size: 12px;
//             margin: 12px 0;
//         }}
//         .talent-entry a {{
//             color: #6db3c6;
//             text-decoration: none;
//         }}
//         .talent-entry a:hover {{
//             text-decoration: underline;
//         }}
//         #results {{
//             min-height: 100px;
//         }}
//         .loading {{
//             text-align: center;
//             padding: 40px;
//             color: #888;
//         }}
//         .error {{
//             color: #e06c75;
//             background: #2a1a1a;
//             padding: 16px;
//             border-radius: 6px;
//             border-left: 4px solid #e06c75;
//         }}
//     </style>
// </head>
// <body>
//     <h1>⚔️ WarcraftLogs Talent Trends</h1>
//
//     <div class="form-container">
//         <form id="talent-form">
//             <select name="region" id="region" required>
//                 {}
//             </select>
//             <select name="encounter" id="encounter" required>
//                 <option value="">Select Boss</option>
//                 {}
//             </select>
//             <select name="class" id="class" required>
//                 <option value="">Select Class</option>
//                 {}
//             </select>
//             <select name="spec" id="spec" required disabled>
//                 <option value="">Select Spec</option>
//             </select>
//             <button type="submit" id="submit-btn" disabled>Get Talents</button>
//         </form>
//     </div>
//
//     <div id="results"></div>
//
//     <script>
//         const specsData = {{
//             {}
//         }};
//
//         const regionSelect = document.getElementById('region');
//         const encounterSelect = document.getElementById('encounter');
//         const classSelect = document.getElementById('class');
//         const specSelect = document.getElementById('spec');
//         const submitBtn = document.getElementById('submit-btn');
//         const resultsDiv = document.getElementById('results');
//
//         regionSelect.addEventListener('change', updateSubmitButton);
//         encounterSelect.addEventListener('change', updateSubmitButton);
//
//         classSelect.addEventListener('change', (e) => {{
//             const selectedClass = e.target.value;
//             specSelect.innerHTML = '<option value="">Select Spec</option>';
//
//             if (selectedClass && specsData[selectedClass]) {{
//                 specsData[selectedClass].forEach(spec => {{
//                     const option = document.createElement('option');
//                     option.value = spec;
//                     option.textContent = spec;
//                     specSelect.appendChild(option);
//                 }});
//                 specSelect.disabled = false;
//             }} else {{
//                 specSelect.disabled = true;
//             }}
//             updateSubmitButton();
//         }});
//
//         specSelect.addEventListener('change', updateSubmitButton);
//
//         function updateSubmitButton() {{
//             const allSelected = regionSelect.value && encounterSelect.value && classSelect.value && specSelect.value;
//             submitBtn.disabled = !allSelected;
//         }}
//
//         document.getElementById('talent-form').addEventListener('submit', async (e) => {{
//             e.preventDefault();
//             const formData = new FormData(e.target);
//             const params = new URLSearchParams(formData);
//
//             resultsDiv.innerHTML = '<h2>Top 10 Talents</h2><div class="loading">Loading talent strings...</div>';
//             submitBtn.disabled = true;
//
//             const eventSource = new EventSource(`/api/talents?${{params}}`);
//             let firstData = true;
//
//             eventSource.onmessage = (event) => {{
//                 if (firstData) {{
//                     resultsDiv.innerHTML = '<h2>Top 10 Talents</h2>';
//                     firstData = false;
//                 }}
//                 resultsDiv.innerHTML += event.data;
//             }};
//
//             eventSource.addEventListener('complete', () => {{
//                 eventSource.close();
//                 updateSubmitButton();
//             }});
//
//             eventSource.onerror = (err) => {{
//                 console.error('EventSource error:', err);
//                 eventSource.close();
//                 if (firstData) {{
//                     resultsDiv.innerHTML = '<div class="error">Connection error. Please try again.</div>';
//                 }}
//                 updateSubmitButton();
//             }};
//         }});
//     </script>
// </body>
// </html>
// "#, region_options, encounter_options, class_options, specs_map)
// }
// / }
