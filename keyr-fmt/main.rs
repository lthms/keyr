/* keyr -- keep track of your keystrokes
 * Copyright (c) 2020 Thomas Letan
 *
 * This file is part of keyr.
 *
 * keyr is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * keyr is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with keyr.  If not, see <https://www.gnu.org/licenses/>.
 */

#[macro_use] extern crate serde_json;

use chrono::{Date, Utc};
use std::io::Result;
use std::fs::OpenOptions;
use std::collections::HashMap;
use serde_json::Value;
use clap::{App, ArgMatches, ArgGroup};
use keyr::{CounterFile, DayFile, GlobalFile, EntryLoc};
use tinytemplate::TinyTemplate;
use num_format::{ToFormattedString, SystemLocale};

enum Output<'a> {
    Json,
    Template(&'a str),
}

enum Details {
    Minimal,
    Summary,
    Full,
}

impl Details {
    fn fetch(&self) -> Result<Value> {
        let mut global = GlobalFile::open(options())?;
        let count = global.read_global_count()?;

        match self {
            Details::Minimal => {
                let date = Utc::now();
                let mut today_file = DayFile::open(&date.date(), options())?;
                let today_count = today_file.read_global_count()?;

                Ok(json!({
                    "global_count": count,
                    "today_count": today_count,
                }))
            },
            Details::Summary => {
                let mut res = HashMap::new();

                keyr::list_days()?
                    .iter()
                    .map(|date| day_summary(&mut res, date))
                    .collect::<Result<Vec<_>>>()?;

                Ok(json!({
                    "global_count": count,
                    "day_counts": res,
                }))
            }
            Details::Full => {
                let mut res : HashMap<i64, u32> = HashMap::new();

                keyr::list_days()?
                    .iter()
                    .map(|date| day_full(&mut res, date))
                    .collect::<Result<Vec<_>>>()?;

                Ok(json!({
                    "global_count": count,
                    "minutes_count": res,
                }))
            },
        }

    }
}

fn options() -> OpenOptions {
    OpenOptions::new()
        .read(true)
        .clone()
}

fn day_summary(map : &mut HashMap<i64, u32>, date : &Date<Utc>) -> Result<()> {
    let mut day_file = DayFile::open(date, options())?;

    let count = day_file.read_global_count()?;

    map.insert(date.and_hms(23, 59, 59).timestamp(), count);

    Ok(())
}

fn day_full(map : &mut HashMap<i64, u32>, date : &Date<Utc>) -> Result<()> {
    let mut day_file = DayFile::open(date, options())?;

    day_file.read_global_count()?;

    loop {
        if let Some((key, count)) = day_file.read_entry(EntryLoc::Next)? {
            if let Some((hour, minute)) = keyr::parse_key(&key) {
                let time = date.and_hms(hour, minute, 59);

                map.insert(time.timestamp(), count);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok(())
}

fn get_app() -> App<'static, 'static> {
    App::new("keyr-fmt")
        .version("0.0.0-dev")
        .author("Thomas Letan <lthms@soap.coffee")
        .about("Format your keystrokes statistics")
        .args_from_usage(
            "--minimal 'The global counter and today’s counter'
             --summary 'The global counter and the list of daily counters'
             --full 'The global counter, and a recap minute per minute'
             --template [string] 'A template to output the result'
             --json 'Output the json as computed'")
        .group(
            ArgGroup::with_name("details")
                .args(&["minimal", "summary", "full"])
        )
        .group(
            ArgGroup::with_name("output")
                .args(&["template", "json"])
        )
}

fn get_cli_args<'a>(matches : &'a ArgMatches<'static>) -> (Details, Output<'a>) {
    let details =
        match (matches.is_present("summary"),
               matches.is_present("full")) {
            (true, _) => Details::Summary,
            (_, true) => Details::Full,
            (_, _)    => Details::Minimal,
        };

    let output =
        match matches.value_of("template") {
            Some(tmp) => Output::Template(tmp),
            _         => Output::Json,
        };

    (details, output)
}

fn num_format_formatter(
    val : &Value,
    output : &mut String
) -> tinytemplate::error::Result<()> {
    match val {
        Value::Number(x) if x.is_i64() => {
            output.push_str(
                &x.as_i64().unwrap()
                    .to_formatted_string(&SystemLocale::default().unwrap())
            );
            Ok(())
        },
        _ => Err(tinytemplate::error::Error::GenericError {
            msg : "`num_format' is for integers only".into(),
        })
    }

}

fn main() -> Result<()> {
    let app = get_app().get_matches();
    let (details, output) = get_cli_args(&app);

    let res = details.fetch()?;

    match output {
        Output::Json => println!("{}", res.to_string()),
        Output::Template(tpl) => {
            let mut tt = TinyTemplate::new();
            tt.add_template("fmt", tpl).unwrap();
            tt.add_formatter("num_format", num_format_formatter);

            println!("{}", tt.render("fmt", &res).unwrap());
        },
    }

    Ok(())
}
