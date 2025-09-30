use crate::config::ClassSpecs;
use crate::warcraftlogs::TalentData;

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
    
    // Build JS object mapping classes to specs
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
    <title>Talent Trends</title>
    <style>
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
        .talent-entry {{ 
            border: 1px solid #444;
            padding: 16px;
            margin: 12px 0;
            border-radius: 6px;
            background: #2a2a2a;
        }}
        .talent-entry h3 {{
            margin-top: 0;
            color: #c69b6d;
        }}
        .talent-string {{ 
            font-family: 'Courier New', monospace;
            background: #1a1a1a;
            padding: 8px;
            border-radius: 4px;
            overflow-x: auto;
            font-size: 12px;
            margin: 12px 0;
        }}
        .talent-entry a {{
            color: #6db3c6;
            text-decoration: none;
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
        }}
        .error {{
            color: #e06c75;
            background: #2a1a1a;
            padding: 16px;
            border-radius: 6px;
            border-left: 4px solid #e06c75;
        }}
    </style>
</head>
<body>
    <h1>⚔️ WarcraftLogs Talent Trends</h1>
    
    <div class="form-container">
        <form id="talent-form">
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
        
        const encounterSelect = document.getElementById('encounter');
        const classSelect = document.getElementById('class');
        const specSelect = document.getElementById('spec');
        const submitBtn = document.getElementById('submit-btn');
        const resultsDiv = document.getElementById('results');
        
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
            const allSelected = encounterSelect.value && classSelect.value && specSelect.value;
            submitBtn.disabled = !allSelected;
        }}
        
        document.getElementById('talent-form').addEventListener('submit', async (e) => {{
            e.preventDefault();
            const formData = new FormData(e.target);
            const params = new URLSearchParams(formData);
            
            resultsDiv.innerHTML = '<div class="loading">Loading top talents...</div>';
            submitBtn.disabled = true;
            
            try {{
                const response = await fetch(`/api/talents?${{params}}`);
                const html = await response.text();
                resultsDiv.innerHTML = html;
            }} catch (err) {{
                resultsDiv.innerHTML = `<div class="error">Error: ${{err.message}}</div>`;
            }} finally {{
                updateSubmitButton();
            }}
        }});
    </script>
</body>
</html>
"#, encounter_options, class_options, specs_map)
}

pub fn render_talents(data: &[TalentData]) -> String {
    if data.is_empty() {
        return r#"<div class="loading">No talent data found.</div>"#.to_string();
    }
    
    let entries: String = data
        .iter()
        .enumerate()
        .map(|(i, t)| {
            format!(
                r#"<div class="talent-entry">
                    <h3>#{} - {}</h3>
                    <div class="talent-string">{}</div>
                    <a href="{}" target="_blank" rel="noopener">View Log →</a>
                </div>"#,
                i + 1, t.name, t.talent_string, t.log_url
            )
        })
        .collect();
    
    format!("<h2>Top 10 Talents</h2>{}", entries)
}



// use crate::config::ClassSpecs;
// use crate::warcraftlogs::TalentData;
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
//     // Build JS object mapping classes to specs
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
//     </style>
// </head>
// <body>
//     <h1>⚔️ WarcraftLogs Talent Trends</h1>
//
//     <div class="form-container">
//         <form id="talent-form">
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
//         const classSelect = document.getElementById('class');
//         const specSelect = document.getElementById('spec');
//         const submitBtn = document.getElementById('submit-btn');
//         const resultsDiv = document.getElementById('results');
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
//                 submitBtn.disabled = true;
//             }}
//         }});
//
//         specSelect.addEventListener('change', (e) => {{
//             submitBtn.disabled = !e.target.value;
//         }});
//
//         document.getElementById('talent-form').addEventListener('submit', async (e) => {{
//             e.preventDefault();
//             const formData = new FormData(e.target);
//             const params = new URLSearchParams(formData);
//
//             resultsDiv.innerHTML = '<div class="loading">Loading top talents...</div>';
//             submitBtn.disabled = true;
//
//             try {{
//                 const response = await fetch(`/api/talents?${{params}}`);
//                 const html = await response.text();
//                 resultsDiv.innerHTML = html;
//             }} catch (err) {{
//                 resultsDiv.innerHTML = `<p style="color: #e06c75;">Error: ${{err.message}}</p>`;
//             }} finally {{
//                 submitBtn.disabled = false;
//             }}
//         }});
//     </script>
// </body>
// </html>
// "#, class_options, specs_map)
// }
//
// pub fn render_talents(data: &[TalentData]) -> String {
//     if data.is_empty() {
//         return r#"<div class="loading">No talent data found.</div>"#.to_string();
//     }
//
//     let entries: String = data
//         .iter()
//         .enumerate()
//         .map(|(i, t)| {
//             format!(
//                 r#"<div class="talent-entry">
//                     <h3>#{} - {}</h3>
//                     <p class="talent-string">{}</p>
//                     <a href="{}" target="_blank" rel="noopener">View Log →</a>
//                 </div>"#,
//                 i + 1, t.name, t.talent_string, t.log_url
//             )
//         })
//         .collect();
//
//     format!("<h2>Top 10 Talents</h2>{}", entries)
// }
