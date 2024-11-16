CONL is a post-minimalist, human-centric configuration language.

It is a replacement for JSON/YAML/TOML, etc... that supports a JSON-like data model of values, maps and lists; but is designed to be much easier to work with.

Consider this [example file](../example.conl):

![Screenshot 2024-11-16 at 00 03 11](https://github.com/user-attachments/assets/395f48b9-f63b-417f-a120-7475c84a9dce)
![Screenshot 2024-11-16 at 00 03 11](https://github.com/user-attachments/assets/4b2a46d5-6702-408e-a6e9-3ddc21ad69d0)



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

A comment begins with a # (U+0023), and continues until the next newline.
To allow keys or values to contain #, comments must usually be preceded
by a blank (or an =, or a close quote).
```
comment = '#' (^ '\r' | 'n')*
```

A quoted scalar begins with a double quote (U+0022) and ends with the same character.
```
quoted_scalar = '"' ( escape | [^ '\\' | '"' | '\r' | '\n'] )* '"'
```

Within quoted scalars, escapes may be used:

```
escape = '\' ( '\' | '"' | 'r' | 'n' | 't' | ( '{' [0-9a-fA-F]{1,8} '}' )
```

A key in CONL may either be a quoted scalar, or (more usually) a non-empty
string of characters that does not start with '"', or contain '=' or ' #' or
'\t#'. Keys may be surrounded by blanks, but within the key blanks are preserved.

```
normal_key = [ ^ '"' | '#' | '=' | blank ] (  [^ '#' | '=' | blank ]+ | blank+ [^ '#' | '='] )*
key = normal_key | quoted_scalar
```

Values are the same as keys, but = characters are also allowed. As with keys, there is no
semantic difference between quoted and unquoted values. "false" and false are equivalent,
and the meaning (a string or a boolean) is determined by the context.

```
normal_value = [ ^ '"' | '#' | blank ] ( [ ^ '#' | blank ]+ | blank+ [^ '#' ] )*
value = normal_value | quoted_scalar
```

For longer values, or values that contain newlines, you can use multline syntax. To allow for better syntax highlighting in modern editors, multiline tokens can be tagged with the expected language.

After parsing, multline tokens have all initial and final blanks and newlines removed. All newlines become \n, and any trailing or leading whitespace on individual lines is preserved. This means they cannot represent values that start or end with blanks or whitespace, or values containing carriage returns.

```
multline_tag = [^ '#' ] ( [^ '#' | blank ]+ | blank+ [^ '#' ] )*
multiline_value = '"""' multiline_tag? blank* comment? newline indent .* outdent
```

Maps and lists are represented as indent-separated sections in the file.
```
section = list_section | map_section
map_section = (map_item | comment? newline)*
list_section = (comment? newline)* (list_item | comment? newline)+
```

Within a section any list item or map key can be set to either a single value, a multiline value, a map or a list. An = sign is allowed (but discouraged) after a map key before a nested section.

```
list_item: '=' blank* any_value
map_item: key blank* blank '=' any_value
        | key blank* (blank comment)? newline (indent section outdent)?

any_value: value blank* (blank comment)? newline
         | multiline_value
         | comment? newline (indent section outdent)?
```

It is allowed for no value to appear after a map key or a list item. As there is no way in CONL to represent a zero-length string, map, or array explicitly. Parsers should treat this as equivalent to whichever is expected.

## Indents

The `level` of a line is the string of tab and space characters at the start. Lines that contain no non-blank characters, or only blanks followed by a comment, are assumed to have the same indentation as the previous line.

Any mix of tabs and spaces is allowed in the `level` and they are considered distinct. Within a multiline string indent/outdent tokens are not generated, so that multiline values can contain inconsistent indentation.

After a newline, there are four possibilities:
* The level of this line matches the previous one. No tokens are generated.
* The level of this line starts with the level of the previous line, and it is longer. In that case an `indent` token is generated.
* The level of this line is shorter than the previous one and matches an earlier line. In this case one `outdent` token is generated per `indent` token generated since that line.
* The level of this line does not match an earlier line. This is an error.

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
