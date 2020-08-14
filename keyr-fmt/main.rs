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

use std::io::Result;
use serde_json::Value;
use clap::{App, ArgMatches, ArgGroup};
use tinytemplate::TinyTemplate;
use num_format::{ToFormattedString, SystemLocale};

use keyr_localstorage as kls;

enum Output<'a> {
    Json,
    Template(&'a str),
}

fn get_app() -> App<'static, 'static> {
    App::new("keyr-fmt")
        .version("0.0.0-dev")
        .author("Thomas Letan <lthms@soap.coffee")
        .about("Format your keystrokes statistics")
        .args_from_usage(
            "--template [string] 'A template to output the result'
             --json 'Output the json as computed'")
        .group(
            ArgGroup::with_name("output")
                .args(&["template", "json"])
        )
}

fn get_cli_args<'a>(matches : &'a ArgMatches<'static>) -> Output<'a> {
    let output =
        match matches.value_of("template") {
            Some(tmp) => Output::Template(tmp),
            _         => Output::Json,
        };

    output
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
    let output = get_cli_args(&app);

    let conn = kls::get_database().unwrap();

    let res = json!({
        "global_count": kls::get_global_count(&conn).unwrap(),
        "today_count": kls::get_today_count(&conn).unwrap(),
    });

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
