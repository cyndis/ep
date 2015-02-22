# ep - Quick and dirty command line EPWING dictionary reader

`ep` is a tiny program for reading EPWING dictionaries; just enough to
use it as a dictionary while translating text.

## Usage

```
Usage: ep [options] <search query>
Options:
  -b <book path>                Specify path to EPWING book. Default is environment variable
                                $EP_BOOK_PATH.
  --expand-unknown-characters   Show character codes for characters without Unicode codepoint.
```
