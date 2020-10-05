use std::collections::HashMap;
use std::fs;

use crate::{Schedule, Day, die};

pub fn generate_html(tmpl_path: String, sched: Schedule) -> String {
    let tmpl = fs::read_to_string(tmpl_path)
                   .unwrap_or_else(|_| die("Could not open calendar template"));

    let mut css = String::new();
    let mut html = String::new();

    let mut event_ctr: u16 = 1;
    for event in sched.events {
        for when in event.when {
            let column = match when.day {
                Day::Monday => 2,
                Day::Tuesday => 3,
                Day::Wednesday => 4,
                Day::Thursday => 5,
                Day::Friday => 6,
                Day::Saturday => 7,
                Day::Sunday => 8,
            };

            let top: f32 = (when.time as f32 - 360.0) / 60.0;
            let height: f32 = when.length as f32 * (5.0 / 3.0);
            let buffer: f32 = top.fract() * 100.0;
            let top: f32 = top.floor();
    
            let (r, g, b) = event.colour;
            
            css.push_str(&format!(".event-{} {{\n", event_ctr));
            css.push_str(&format!("    background-color: rgb({}, {}, {});\n", r, g, b));
            css.push_str(&format!("    grid-column: {};\n", column));
            css.push_str(&format!("    grid-row: {};\n", top + 2.0));
            css.push_str(&format!("    top: {}%;\n", buffer));
            css.push_str(&format!("    height: calc({}% - 8px);\n", height));
            css.push_str("}\n\n");

            html.push_str(&format!("<div class=\"event event-{}\">\n", event_ctr));
            html.push_str(&format!("    {}\n", event.title));
            html.push_str("</div>\n");

            event_ctr += 1;
        }
    }

    let doc = tmpl.replace("%%STYLE%%", &css);
    let doc = doc.replace("%%HTML%%", &html);

    doc
}
