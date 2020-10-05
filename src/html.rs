use std::collections::HashMap;
use std::fs;

use crate::{Schedule, Day, die};

pub fn generate_html(tmpl_path: String, sched: Schedule, cat_index: HashMap<String, usize>) -> String {
    let tmpl = fs::read_to_string(tmpl_path)
                   .unwrap_or_else(|_| die("Could not open calendar template"));

    let mut css = String::new();
    let mut html = String::new();

    for (_, id) in cat_index.iter() {
        let category = sched.categories.get(*id).expect("Assertion Failed: No such category");
        css.push_str(&format!(".cat-{} {{\n", id));
        let (r, g, b) = category.colour;
        css.push_str(&format!("    background-color: rgb({}, {}, {});\n", r, g, b));
        css.push_str("}\n\n");
    }

    let mut event_ctr: u16 = 1;
    for event in sched.events {
        for when in event.when {
            let column = match when.day {
                Day::Monday => 2,
                Day::Tuesday => 3,
                Day::Wednesday => 4,
                Day::Thursday => 5,
                Day::Friday => 6,
                Day::Saturday => die("Web output is weekdays only"),
                Day::Sunday => die("Web output is weekdays only"),
            };

            let top: f32 = (when.time as f32 - 360.0) / 60.0;
            let height: f32 = when.length as f32 * (5.0 / 3.0);
            let buffer: f32 = top.fract() * 100.0;
            let top: f32 = top.floor();

            css.push_str(&format!(".event-{} {{\n", event_ctr));
            css.push_str(&format!("    grid-column: {};\n", column));
            css.push_str(&format!("    grid-row: {};\n", top + 2.0));
            css.push_str(&format!("    top: {}%;\n", buffer));
            css.push_str(&format!("    height: calc({}% - 8px);\n", height));
            css.push_str("}\n\n");

            html.push_str(&format!("<div class=\"cat-{} event event-{}\">\n", event.category, event_ctr));
            html.push_str(&format!("    {}\n", event.title));
            html.push_str("</div>\n");

            event_ctr += 1;
        }
    }

    let doc = tmpl.replace("%%STYLE%%", &css);
    let doc = doc.replace("%%HTML%%", &html);

    doc
}
