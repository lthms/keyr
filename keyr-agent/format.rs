use anyhow::Result;
use serde_json::Value;
use tinytemplate::TinyTemplate;
use num_format::{ToFormattedString, SystemLocale};

use keyr_agentstorage as kas;
use kas::SqliteConnection;

use crate::cli::Output;

fn num_format_formatter(
    val : &Value,
    output : &mut String
) -> tinytemplate::error::Result<()> {
    match val {
        Value::Number(x) if x.is_i64() => {
            output.push_str(
                // FIXME
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

pub fn run(conn : &SqliteConnection, output : &Output) -> Result<()> {
    let res = json!({
        "global_count": kas::get_global_count(&conn)?,
        "today_count": kas::get_today_count(&conn)?,
    });

    match output {
        Output::Json => println!("{}", res.to_string()),
        Output::Template(tpl) => {
            let mut tt = TinyTemplate::new();
            tt.add_template("fmt", tpl)?;
            tt.add_formatter("num_format", num_format_formatter);

            println!("{}", tt.render("fmt", &res)?);
        },
    }

    Ok(())
}
