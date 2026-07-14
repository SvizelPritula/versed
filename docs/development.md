# The developer's guide

## The documentation

Versed uses `rustdoc` to build its documentation.
To build the documentation, simply run the following command in the root of the repo:

```sh
cargo doc --document-private-items
```

To view the documentation, simply open `target/doc/versed/index.html`
in your browser afterwards.
