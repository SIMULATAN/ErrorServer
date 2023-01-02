# ErrorServer
[![Screenshot](screenshot.png)](screenshot.png)
:rocket: A simple & lightweight server that returns a HTML page of the error code with its respective message and debug information, written in rust :crab:

## Features
- automatic resolving of error codes to their respective messages
  - if the error code is not found, it will print a generic error message
- simple, but not eye bleeding design
  - dark mode per default
  - colors follow the dracula theme specification
- display debug information
  - the whole request is printed in the debug section
  - hidden behind a dropdown to prevent accidentally leaking sensitive information
- perfect for traefik (config coming soon!)
- written in rust :crab: :rocket:
- low-memory footprint
  - ~ 1 MB while idling, benchmarks will follow
- small binary size
  - ~ 400 KB
- low-dependency
  - apart from the rust standard library, only one dependencies is used (`signal-hook`)
- easy to use
  - just run the binary and you're good to go
- docker support
  - built & published for AMD64 and ARM64

## Contributions
... are welcome! If you have any ideas, feel free to open an issue or a pull request. I'd also appreciate feedback on the code, as I'm still learning rust (this is literally my first project using it :sweat_smile:).
