use diesel_migrations::{MigrationConnection, RunMigrationsError};

embed_migrations!();

pub fn run<Conn>(connection : &Conn) -> Result<(), RunMigrationsError>
where Conn : MigrationConnection {
    embedded_migrations::run(connection)
}
