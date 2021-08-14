# RHC

**R**ijks **H**tml **C**oncatenator.

A simple markup format to combine HTML content from multiple files.
The webserver uses the `.rhc` extension the differentiate between files.

NOTE: This format is used for on my website and should not be used in real products.

## Markup reference

Sections are marked with `{` and `}`. The sections will be replaced with other content.

## Keys

The value of keys can be set in code.

`{#my_key}`

## Files

The path is allways relative to the file.

`{@inc/header.rhc}`
`{@footer.rhc}`

