embed_migrations!();

use diesel::pg::Pg;
use diesel::prelude::*;

pub fn run<Conn>(conn : &Conn) -> crate::error::Result<()>
where
    Conn : Connection<Backend = Pg>,
{
    embedded_migrations::run(conn)?;

    Ok(())
}
