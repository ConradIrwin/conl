CONL is a post-minimal, human-centric configuration language.

It is designed as a drop-in replacement for JSON/INI/YAML/TOML/etc... for when you want a file that users are comfortable editing by hand.

Features:

* A JSON-like data model of maps, lists and scalars.
* An INI-like type system, where type inference is deferred until parse time.
* A (simplified) YAML-like syntax, where indentation defines structure.

Design criteria:

* Easy to read. CONL files parse unambiguously to both humans and machines.
* Easy to edit. No need to spend time balancing brackets.
* JSON-iteroperable. You can validate a CONL document with JSON schema if you want.
* Easy to parse. Building a CONL parser is a fun afternoon, not a week of deciphering edge-cases.

Consider this [example file](./example.conl):

<img width="700" alt="Screenshot 2024-11-17 at 23 27 14" src="https://github.com/user-attachments/assets/adb36d3b-b9fe-4c85-857a-db55aff36d2d">
<img width="700" alt="Screenshot 2024-11-17 at 23 27 48" src="https://github.com/user-attachments/assets/a71c415e-c836-40e4-a52c-f9eb6fef127f">

## Syntax

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

If your value contains newlines (or `;`) use multiline syntax. The syntax
highlighting hint is optional and is not parsed, but gives you nice highlighting
in modern editors.

```conl
init_script = """bash
  #!/bin/bash

  echo "hello world";
```

In the rare case you need more flexiblity, for example to preserve
leading/trailing whitespace, or to represent the presence of the empty string,
you can use a quoted literal:

```conl
"" = " wow\r\nlook at my cat \{1F431}"
```

Quoting is not meaningful. CONL uses deferred typing to ensure that the
application receives the type it needs without users having to remember the
syntax.
```conl
enabled = true
enabled = "true" ; equivalent to enabled = true
enabled = """    ; equivalent to enabled = true
   true
```

This also allows for a richer set of types. Instead of downcasting to just
numbers, you can have fields that expect specific units without adding more
syntactic noise:

```conl
max_space = 10 GB
timeout = 10s
country_code = no
```

CONL uses `;` for comments. This means that common values like HTML colors don't require quotes.
```conl
color = #ff0000 ; pure red
```

Any JSON/YAML/TOML document can be converted to CONL, but the reverse (to do it
propertly) requires a schema to ensure the correct type-syntax is generated
(or you can guess ;).

If you'd like to build your own implementation, [spec.conl](./spec.conl)
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
