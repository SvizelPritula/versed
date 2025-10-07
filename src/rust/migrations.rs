use std::io::{Result, Write, stdout};

use crate::{
    ast::Migration, migrations::pair_types, preprocessing::BasicMetadata, rust::convert_types,
};

pub fn generate_migration(migration: Migration<BasicMetadata>) -> Result<()> {
    let migration = migration.map(convert_types);

    let pairs = pair_types(&migration);

    writeln!(stdout().lock(), "{pairs:#?}")
}
