CONL is a post-minimalist, human-centric configuration language. It is a replacement for JSON/YAML/TOML, etc... that supports a JSON-like data model of values, maps and lists; but is designed to be much easier to work with.

Taking retro-inspriation from INI files, CONL does not have different syntax for different types of values. The type of a value is determined by the context, and so if a parser expects an int64 it knows to accept a `1` but reject a `1.1`; or if it expects a country code it can accept `no` for Norway, but reject `yes`.

To allow for easy commenting out of sections of the file, there are no brackets, braces or commas. The structure is always defined by indentation and values are not quoted (though there are escape sequences for esoteric use cases). To simplify the user-experience, there is no equivalent of JSON's `null`: default values should be omitted or commented out.

## Overview

A simple CONL document showing how structure is represented by newlines, indentation and `=` signs:

```
# comments start with # and continue until the next newline
# There are four possible ways to set a value:
key = value

map
   a = b
   c = d

list
   = a
   = b

multiline_value = """
   multiline
   token
```

Some more complicated things:

```
# multiline strings can be tagged with a language to get syntax highlighting.
# Neither comments nor escape sequences are processed within them
init = """bash
   #!/bin/bash
   echo "hello world"

# spaces are allowed within both keys and values
key with spaces = value with spaces

# Lists can of course also contain maps (and so on all the way down)
# json: {"list_of_maps": [{"a": "b"}, {"b": "c"}]}
list_of_maps
  =
    a = b
  =
    b = c

# "X defines escape characters when you need them.
#
# Escapes are only needed for:
# * quotes (use "")
# * carriage returns or newlines (use "\ or "/)
# * leading or trailing spaces or tabs (use "_ or ">)
# * an = or # following a space or tab in a key (use "# or "=)
# * a # following a space or a tab in a value ("# again)
# * representing the empty string ("@)
# That said, you can use them to reference any hard to type character
# by codepoint ("{XXXX} gives you U+XXXX).
#
# json: {"key with \"=\" in it ": "value with \t in it (null-terminated)\u0000"}
key with """="" in it"_ = value with "> in it (null-terminated)"{0}
```

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
* `"@` generates an empty string. This is only useful when converting a JSON document that may contain empty map keys, or empty list items to CONL. Empty map values are conventionally omitted.
* `"{ [0-9a-fA-F]+ }` generates the unicode character with the specified hexadecimal value. Unpaired surrogates are disallowed to ensure that all values are valid UTF-8.
```
escape = '"' | '#' | '=' | '_' | '>' | '\' | '/' | '@' | ( '{' [0-9a-fA-F]+ '}' )
```

A key in CONL always starts and ends with a non-blank, non-newline character. Within a key blanks are preserved. The characters = and # may be included in a key if they are either escaped, or not preceded by blanks.

```
key_char = (^ ' ' | '\t' | '\r' | '\n' | '"' | '#' | '=') | ('"' escape)
key = key_char (key_char | '#' | '=' | blank+ key_char)*
```

Values are the same as keys, but = characters are always allowed.

```
value_char = (^ ' ' | '\t' | '\r' | '\n' | '"' | '#') | ('"' escape)
value = value_char (value_char | '#' | blank+ value_char)*
```

For longer values, or values that contain newlines, you can use multline syntax. To allow for better syntax highlighting in modern editors, multiline tokens can be tagged with the expected language. To avoid ambiguity with escapes, the language tag must start with an ASCII letter or number.

After parsing, multline tokens have all leading and trailing blanks and newlines removed, and all newlines are normalized to a newline character. This means they cannot represent values with leading or trailing blanks or whitespace, or values containing carriage returns.

```
multline_tag = [a-zA-Z0-9] value
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

CONL cannot explicitly represent a `null` value (to avoid the unnecessary distinction between a key mapped to null and a missing key). For maps you should omit keys that have the default value, and for list items (or map keys) you can use the empty string `"@`.

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
