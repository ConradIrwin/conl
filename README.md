CONL is a post-minimalist, human-centric configuration language.

It supports a JSON-like data model of values, maps and lists; but with a syntax that ensures common operations like commenting out a line, or adding a new key/value pair always leave the document in a parseable state.

Consider this [example file](../example.conl):

## Syntax

The syntax of CONL has been designed with several priorities (in order):

1. To be easy to read
3. To be easy to edit
2. To be round-trippable to JSON
4. To be easy to parse

CONL uses indentation for structure. This provides an advantage over JSON in that common operations (like commenting out a line) do not make the document invalid, and an advantage over TOML/INI in that you can construct arbitrary list/map nestings instead of just "tables".

```conl
map
  key1 = value1
  key2 = value2
  ;...

list
  = value1
  = value2
```

Without any special syntax, keys can contain any character except `;`, `=`, `\r`, `\n` and values can contain any character except `;`, `\r`, `\n`.

```conl
key containing spaces = value with = and "quotes"!
```

If your value contains newlines (or ;) use multiline syntax. The syntax highlighting hint is optional and is not parsed, but gives you nice highlighting in modern editors.

```conl
init_script = """bash
  #!/bin/bash

  echo "hello world";
```

In the rare case you need more flexiblity, for example to preserve leading/trailing whitespace, or to represent the empty string, you can use a quoted literal:

```conl
"" = " wow\r\nlook at my cat \{1F431}"
```

Quoting is not meaningful. CONL uses deferred typing to ensure that the
application receives the type it needs without users having to remember the
syntax.
```conl
enabled = "true" ; equivalent to enabled = true
enabled = """    ; equivalent to enabled = true
   true
```

This also allows for a richer set of types. Instead of downcasting to just
numbers, you can have fields that expect specific units without adding more
syntactic noise:

```conl
max_space = 10GB
timeout = 10s
country_code = no
```

CONL uses `;` for comments. This means that common values like HTML colors don't require quotes.
```conl
color = #ff0000 ; pure red
```

Any JSON/YAML/TOML document can be converted to CONL, but the reverse requires a schema to ensure the correct types are generated.

If you'd like to build your own implementation, [spec.conl](../spec.conl)
contains a relatively complete specification of the syntax.

# Why?

Why not? I was inspired to create CONL by this excellent [INI critique of
TOML](https://github.com/madmurphy/libconfini/wiki/An-INI-critique-of-TOML). It
reminded me that my struggles to write TOML or YAML by hand were not due to
failings on my part, but due to the inherent complexity of a "minimal" format
that has four syntaxes for strings, and eleven other data-types to contend with.

In my day-to-day life I spend a non-trivial amount of time editing configuration
files that are either giant chunks of YAML (Github workflows, Kubernetes
manifests...), giant chunks of JSON-with-comments files (Zed's configuration
files), or TOML (Rust cargo files). What if there were one format that could do
it all, and do it all in a relatively easy way.
