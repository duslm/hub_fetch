# hub_fetch

Simple program that will download the latest release of whatever github repo. 

hub.toml holds all the info about the repo you want, syntax as below.
[packages.{repo}]
  arch = "{arch}"
  file_type = "{filetype}"
  user = "{user}"
  repo = "{repo}"
  get_package = true/false
  get_source = true/false

Future goals:
    better error handling
    flexible versioning (for instance with cargo.toml dependencies)
    inferring arch/filetype from system specs
    match analogous archs/filetypes, eg amd64 == x86_64
    hook a shell script or something to actually install the program