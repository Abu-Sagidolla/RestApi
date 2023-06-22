use diesel::migration::migration_directory;
use diesel::migration::Config as MigrationConfig;
use diesel_migrations::DieselMigrations;

pub fn run_migrations() {
    let mut migration_config = MigrationConfig::new();
    migration_config.migrations_directory = Some(migration_directory().unwrap());

    let mut migration = DieselMigrations::new(migration_config);
    migration.run().unwrap();
}