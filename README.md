CONL is a post-minimalist, human-centric configuration language.

It is a replacement for JSON/YAML/TOML, etc... that supports a JSON-like data model of values, maps and lists; but is designed to be much easier to work with.

Consider this [example file](./example.conl):

<img width="700" alt="Screenshot 2024-10-31 at 00 06 28" src="https://github.com/user-attachments/assets/840ffb35-e369-49f9-9a9e-f6092fb6a956">
<img width="700" alt="Screenshot 2024-10-31 at 00 06 40" src="https://github.com/user-attachments/assets/ec00b8f6-1ba7-4db8-aacd-8e153f7ab7dc">

## Syntax

The syntax of CONL has been designed with several priorities (in order):

1. To be easy to read
2. To be easy to edit
3. To be easy to parse

The source must be valid UTF-8, and because CONL is indentation sensitive this grammar assumes the synthetic `indent` and `outdent` tokens are generated as described below.

In keeping with tradition, a newline may be specified with either a newline (U+000A) or carriage return (U+000D), or both:
```
newline = '\r' | '\n' | '\r\n'
```

Within a line, you can use tabs (U+0009) or spaces (U+0020) for blanks. Other unicode spaces (e.g. U+200B) are treated as any other character (so that parsing is not dependent on Unicode version or multibyte characters).
```
blank = ' ' | '\t'
```

A comment begins with the pound sign (U+0023), and continues until the next newline.
To allow for keys or values that contain a literal pound sign, comments that do not start
at the beginning of a line or after an = must be preceded by a blank.
```
comment = '#' (^ '\r' | 'n')*
```

An escape sequence begins with a double quote (U+0022) and is followed by either a named
escape, or a hexadecimal sequence.
* `""`, `"#`, `"=` generate `"`, `#` and `=` respectively.
* `"_`, `">`, `"\` and `"/` generate space, tab, carriage return and newline.
* `"{ [0-9a-fA-F]+ }` generates the unicode character with the specified hexadecimal value. Unpaired surrogates are disallowed to ensure that all values are valid UTF-8.
```
escape = '"' | '#' | '=' | '_' | '>' | '\' | '/' | ( '{' [0-9a-fA-F]+ '}' )
```

To represent the empty string, you can use `"{}`.
```
empty = `"{}`
```

A key in CONL always starts and ends with a non-blank, non-newline character. Within a key blanks are preserved. The character # may be included in a key if it is escaped, or not preceded by blanks. The character = may be included in a key if it is escaped.

```
key_char = (^ ' ' | '\t' | '\r' | '\n' | '"' | '#' | '=') | ('"' escape)
key = empty | ( key_char (key_char | '#' | blank+ key_char)* )
```

Values are the same as keys, but = characters are also allowed.

```
value_char = (^ ' ' | '\t' | '\r' | '\n' | '"' | '#') | ('"' escape)
value = empty | ( value_char (value_char | '#' | blank+ value_char)* )
```

For longer values, or values that contain newlines, you can use multline syntax. To allow for better syntax highlighting in modern editors, multiline tokens can be tagged with the expected language. Language tags cannot start with an escape sequence to avoid ambiguity, and also may not contain quotes or space to help avoid accidental errors.

After parsing, multline tokens have all initial and final blanks and newlines removed. All newlines become \n, and any trailing or leading whitespace on individual lines is preserved. This means they cannot represent values that start or end with blanks or whitespace, or values containing carriage returns.

```
multline_tag = (^ '"' | '#' | '=' | '_' | '>' | '\' | '/' |  '{') (^ '"' | ' ' | '\t')*
multiline_value = '"""' multiline_tag? blank* comment? newline indent .* outdent
```

Maps and lists are represented as indent-separated sections in the file. A section that contains no items (and for which the parser has no type hints) is considered an empty map. Keys must be unique within a map section.
```
section = list_section | map_section
map_section = (map_item | comment? newline)*
list_section = (comment? newline)* (list_item | comment? newline)+
```

Within a section any list item or map key can be set to either a single value, a multiline value, a map or a list. An = sign is allowed (but discouraged) after a map key before a nested section.

```
list_item: '=' blank* any_value
map_item: key blank* blank '=' any_value
        | key blank* (blank comment)? newline indent section outdent

any_value: value blank* (blank comment)? newline
         | multiline_value
         | comment? newline indent section outdent
```

## Indents

The `level` of a line is the string of tab and space characters at the start. Lines that contain no non-blank characters are assumed to have the same indentation as the previous line, though lines that contain just a comment must have the correct indentation.

Any mix of tabs and spaces is allowed in the `level` and they are considered distinct. Within a multiline string indent/outdent tokens are not generated, so that multiline values can contain inconsistent indentation.

After a newline, there are four possibilities:
* The level of this line matches the previous one. No tokens are generated.
* The level of this line starts with the level of the previous line, and it is longer. In that case an `indent` token is generated.
* The level of this line is shorter than the previous one and matches an earlier line. In this case one `outdent` token is generated per `indent` token generated since that line.
* The level of this line does not match an earlier line. This is an error.

# Other considerations

CONL cannot explicitly represent a `null` value (to avoid the unnecessary distinction between a key mapped to null and a missing key). For maps you should omit keys that have the default value, and for list items (or map keys) you can use the empty string `"{}`.

This means that you cannot distinguish between a `vec![None]` and a `vec![Some("")]` in a map or a list. (Though hopefully such an subtle distinction doesn't make an impact on your application's behaviour)

CONL can represent maps with any key type (not just strings) by parsing the keys as you would values.

Most values can be serialized as either a single-line or a multi-line string. The exceptions are those that start or end ' ', '\t' or '\n', or contain '\r'. Parsers should not distinguish between single-line or multi-line syntax (the indicator is purely for syntax highlighting). Serializers should chose the most convenient (typically if the string contains newlines and can be represented as such, a multiline string is better).

# Why?

Why not? I was inspired to create CONL by this excellent [INI critique of
TOML](https://github.com/madmurphy/libconfini/wiki/An-INI-critique-of-TOML). It
reminded me that my struggles to write TOML or YAML by hand were not due to
failings on my part, but due to the inherent complexity of a "minimal" format
that has four syntaxes for strings, and eleven other data-types to contend with.

In my day-to-day life I spend a non-trivial amount of time editing configuration
files that are either giant chunks of YAML (Github workflows, Kubernetes
manifests...), giant chunks of JSON-with-comments files (Zed's configuration
files), or TOML (Rust cargo files). What if there were one format that married
could do it all? By removing all that is unnecessary only the useful remains.
