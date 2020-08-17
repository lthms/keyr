embed_migrations!();

use diesel::prelude::*;
use diesel::pg::Pg;

pub fn run<Conn>(
    conn : &Conn
) -> crate::error::Result<()>
where Conn : Connection<Backend = Pg> {
    embedded_migrations::run(conn)?;

    Ok(())
}
