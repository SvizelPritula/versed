# The Versed compiler usage guide

There are two commands related to type generation,
`versed rust types` and `versed typescript types`.
Both take two arguments, the path to the schema and the path to the output directory,
in that order.
Versed will write the generated types to a new file inside the output directory
named after the version of the schema.
For example, if you run `versed rust types schema.vs src/schema/`
and `schema.vs` starts with `version v1;`,
then the types will be written to `src/schema/v1.rs`.
It will also add an import of this file to `mod.rs` or `index.ts`.
The directory will be created if it doesn't exist.

You can also use the `-f` or `--to-file` flag to write the types directly to a specified file,
which will cause the second argument to be interpreted as the path to that file
instead of a directory.
For example, `versed rust types schema.vs -f src/current-schema.rs` will simply
write the types to `src/current-schema.rs`.

If you only want to check if a schema file is syntactically and semantically well-formed,
you can use `versed check`.
There is also `versed version`, which will additionally
output the version of the schema.

As for migrations, there are two commands used for creating a migration file
and one for generating the migration functions.
The `versed migration begin` command starts the interactive migration.
It takes the path to the schema file containing the old version,
saves a copy of it and adds migration markers to it.
You can then edit the file as you wish.
The `versed migration finish` command ends the migration
and creates the migration file.
Lastly, `versed rust migration` works like `versed rust types`,
except it generates migration functions instead of type declarations.
For example, you could use `versed migration begin schema.vs` to start the migration,
`versed migration finish schema.vs schema.vsm` to end it
and `versed rust migration schema.vs src/schema/` to create the migration functions.

There is also a `versed migration check` command that corresponds to `versed check`.

Lastly, there is `versed completions`, which prints out a script for providing tab-completion
for `versed` for the specified shell.
For example, you can install tab-completions for bash like this:

```sh
versed completions bash > ~/.local/share/bash-completion/completions/versed
```

You can run `versed help` for a more detailed usage description.
