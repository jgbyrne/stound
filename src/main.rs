use std::io::{self, Read};
use std::fs;
use std::process;
use std::env;
use std::str::FromStr;
use std::collections::HashMap;

use serde;
#[macro_use]
use serde_derive::Deserialize;
use toml;

// Deserialization Data Structures

#[derive(Debug, Deserialize)]
struct ScheduleWhen {
    day:    Option<String>,
    time:   Option<String>,
    length: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScheduleEntry {
    title:    String,
    category: Option<String>,
    day:      Option<String>,

    time:   Option<String>,
    length: Option<String>,
    when:   Option<Vec<ScheduleWhen>>,
}

#[derive(Debug, Deserialize)]
struct CategoryColour {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Deserialize)]
struct CategoryDesc {
    name: String,
    colour: CategoryColour, 
}

#[derive(Debug, Deserialize)]
struct ScheduleFile {
    schedule: Option<Vec<ScheduleEntry>>,
    category: Option<Vec<CategoryDesc>>,
}

// Internal Data Structure 

#[derive(Debug)]
struct Category {
    name: String,
    colour: (u8, u8, u8),
}

#[derive(Debug)]
enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl FromStr for Day {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "monday" => Day::Monday,
            "tuesday" => Day::Tuesday,
            "wednesday" => Day::Wednesday,
            "thursday" => Day::Thursday,
            "friday" => Day::Friday,
            "saturday" => Day::Saturday,
            "sunday" => Day::Sunday,
            _ => { return Err(()) },
        })
    }
}

#[derive(Debug)]
struct When {
    day: Day,
    time: u16,
    length: u16,
}

#[derive(Debug)]
struct Event {
    title: String,
    category: usize,
    when: Vec<When>,
}

#[derive(Debug)]
struct Schedule {
    categories: Vec<Category>,
    events: Vec<Event>,
}

fn die(msg: &str) -> ! {
    eprintln!("Fatal Error: {}", msg);
    process::exit(1);
}

fn parse_day(day: &str) -> Day {
    Day::from_str(day).unwrap_or_else(|_| die("Invalid or missing day"))
}

fn parse_time(time: &str) -> u16 {
    let parts: Vec<&str> = time.split(':').collect();

    match parts.len() {
        1 => {
            parts[0].parse::<u16>().unwrap_or_else(|_| die("Invalid hour")) * 60
        },
        2 => {
            let hour = parts[0].parse::<u16>().unwrap_or_else(|_| die("Invalid hour"));
            let minute = parts[1].parse::<u16>().unwrap_or_else(|_| die("Invalid minute"));
            (hour * 60) + minute
        },
        _ => {
            die("Invalid time - should be hh:mm");
        },
    }
}

fn parse_length(length: &str) -> u16 {
    let mut numbuf = String::new();
    let mut acc: u16 = 0;

    for c in length.chars() {
        match c {
            '0' ..= '9' =>  numbuf.push(c),
            'h' => {
                acc += numbuf.parse::<u16>()
                             .unwrap_or_else(|_| die("Invalid length")) * 60;
                numbuf = String::new();
            },
            'm' => {
                acc += numbuf.parse::<u16>()
                             .unwrap_or_else(|_| die("Invalid length"));
                numbuf = String::new();
            },
            _ => {
                die("Invalid number in length");
            }
        }
    }

    acc
}

fn main()  {
    let mut buf = String::new();

    for file in env::args().skip(1) {
        let mut f = fs::File::open(&file).unwrap_or_else(
                                          |_| die(&format!("Could not open file '{}'", &file)));
        f.read_to_string(&mut buf).unwrap_or_else(
                                   |_| die(&format!("Could not read from file '{}'", &file)));
        buf.push('\n');
    }

    let file: ScheduleFile = toml::from_str(&buf).unwrap();

    let (sched, cat_ids): (Schedule, HashMap<String, usize>)= {
        let mut cat_ids: HashMap<String, usize> = HashMap::new();

        let mut categories = file.category.unwrap_or(vec![]).into_iter().enumerate().map(|(id, c)| {
            cat_ids.insert(c.name.clone(), id);
            Category { name: c.name, colour: (c.colour.r, c.colour.g, c.colour.b) }
        }).collect::<Vec<Category>>();

        let mut events = Vec::new();

        let none = String::from("None");
        if !cat_ids.contains_key(&none) {
            let id = categories.len();
            cat_ids.insert(none.clone(), id);
            categories.push(Category { name: none.clone(), colour: (200, 200, 200) });
        }

        for entry in file.schedule.unwrap_or(vec![]).into_iter() {
            let title = entry.title;
            let category = match entry.category {
                Some(c) => {
                    match cat_ids.get(&c) {
                        None => {
                            let id = categories.len();
                            cat_ids.insert(c.clone(), id);
                            categories.push(Category { name: c.clone(), colour: (50, 50, 50) });
                            id
                        },
                        Some(id) => {
                            *id
                        },
                    }
                },
                None => *cat_ids.get(&none).expect("Assertion Failed: No None Category"),
            };

            let empty = String::new();
            
            let day_str = entry.day.as_ref().unwrap_or(&empty);
            let time_str = entry.time.as_ref().unwrap_or(&empty);
            let length_str = entry.length.as_ref().unwrap_or(&empty);
            
            let when = if let Some(when) = entry.when {
                let mut whens = vec![];
                
                for w in when {
                    whens.push(When {
                        day: parse_day(w.day.as_ref().unwrap_or(day_str)),
                        time: parse_time(w.time.as_ref().unwrap_or(time_str)),
                        length: parse_length(w.length.as_ref().unwrap_or(length_str)),
                    });
                }

                whens
            }
            else {
                let when = When {
                    day: parse_day(day_str),
                    time: parse_time(time_str),
                    length: parse_length(length_str),
                };

                vec![when]   
            };

            events.push(Event {
                title,
                category,
                when,
            });
        }

        (Schedule { categories, events }, cat_ids)
    };

    let tmpl = fs::read_to_string("cal.html")
                   .unwrap_or_else(|_| die("Could not open calendar template"));

    let mut css = String::new();
    let mut html = String::new();

    for (cat, id) in cat_ids.iter() {
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

    println!("{}", doc);
}


