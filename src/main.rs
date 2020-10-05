use std::io::Read;
use std::fs;
use std::process;
use std::env;
use std::str::FromStr;
use std::collections::HashMap;

use serde::Deserialize;
use toml;

mod html;

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
    colour: Option<Colour>,

    day:      Option<String>,
    time:   Option<String>,
    length: Option<String>,
    when:   Option<Vec<ScheduleWhen>>,
}

#[derive(Debug, Deserialize)]
struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Deserialize)]
struct CategoryDesc {
    name: String,
    colour: Colour, 
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
pub struct When {
    day: Day,
    time: u16,
    length: u16,
}

#[derive(Debug)]
pub struct Event {
    title: String,
    category: usize,
    colour: (u8, u8, u8),
    when: Vec<When>,
}

#[derive(Debug)]
pub struct Schedule {
    categories: Vec<Category>,
    events: Vec<Event>,
}

pub fn die(msg: &str) -> ! {
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
    let mut tmpl_path: Option<String> = None;
    let mut debug = false;

    let mut state = '.';
    for arg in env::args().skip(1) {
        match state {
            '.' => {
                match arg.as_str() {
                    "--template" => {
                        state = 't';
                    },
                    "--debug" => {
                        debug = true;
                    },
                    _ => {
                        let mut f = fs::File::open(&arg).unwrap_or_else(
                                                         |_| die(&format!("Could not open file '{}'", &arg)));
                        f.read_to_string(&mut buf).unwrap_or_else(
                                                   |_| die(&format!("Could not read from file '{}'", &arg)));
                        buf.push('\n');
                    },
                }
            },
            't' => {
                tmpl_path = Some(String::from(arg));
                state = '.';
            }
            _ => { unreachable!() }
        }
    }
    if state != '.' { die("Dangling argument") }

    let file: ScheduleFile = match toml::from_str(&buf) {
        Ok(file) => file,
        Err(err) => {
            if debug {
                eprintln!("Internal `serde` error for debugging purposes - line numbers may be wrong!");
                eprintln!("{:?}", err);
            }
            die("TOML Parsing Error");
        },
    };

        

    let sched = {
        let mut cat_index: HashMap<String, usize> = HashMap::new();

        let none = String::from("None");
        let mut categories = file.category.unwrap_or(vec![]).into_iter().enumerate().map(|(id, c)| {
            cat_index.insert(c.name.clone(), id);
            Category { name: c.name, colour: (c.colour.r, c.colour.g, c.colour.b) }
        }).collect::<Vec<Category>>();

        let mut events = Vec::new();

        let (none_id, none_colour) = {
            match cat_index.get(&none) {
                Some(id) => {
                    (*id, categories.get(*id).expect("Inconsistent Category Index").colour)
                },
                None => {
                    let id = categories.len();
                    cat_index.insert(none.clone(), id);
                    categories.push(Category { name: none.clone(), colour: (200, 200, 200) });
                    (id, (200, 200, 200))
                }
            }
        };

        for entry in file.schedule.unwrap_or(vec![]).into_iter() {
            let title = entry.title;
            let (category, cat_colour) = match entry.category {
                Some(c) => {
                    match cat_index.get(&c) {
                        None => {
                            let id = categories.len();
                            cat_index.insert(c.clone(), id);
                            categories.push(Category { name: c.clone(), colour: (50, 50, 50) });
                            (id, (50, 50, 50))
                        },
                        Some(id) => {
                            (*id, categories.get(*id).expect("Inconsistent Category Index").colour)
                        },
                    }
                },
                None => (none_id, none_colour),
            };

            let colour = match entry.colour {
                Some(colour) => (colour.r, colour.g, colour.b),
                None => cat_colour,
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
                colour,
                when,
            });
        }

        Schedule { categories, events }
    };

    print!("{}", html::generate_html(tmpl_path.unwrap_or(String::from("cal.html")), sched));
}
