## intro
`dots` manages your dotfiles through the concept of a "store" which is where
all of your dotfiles live. This store can be safely checked into version
control, moved around, etc. Dotfiles are installed by symlinking from their
source locations in the store to their target in your home directory.

An illustration to clarify the source to target mapping:
| source path           | target path          |
|-----------------------|----------------------|
| `<store path>/<name>` | `<home dir>/.<name>` |

Note that source files in your dotfile store don't have the `.` prefix making
them visible by default!

## tips

### managing multiple stores
You can easily manage multiple stores through the use of the `--store-dir`
flag. I personally have my work dotfiles aliased so that I can easily manage
both my personal and work dotfiles at once:
```
alias sdots='dots --store-dir "~/.dotfiles.stripe"'
```

### listing all files for scripting
```
dots list | sed 's/\[.\] //' | sed 's/(.*)$//'
```

### linking all files in the store
```
dots list | sed 's/\[.\] //' | sed 's/(.*)$//' | xargs -I % -- dots link %
```

### linking files interactively with `fzf`
```
dots list | fzf -m | sed 's/\[.\] //' | sed 's/(.*)$//' | xargs -I % -- dots link %
```

## help text
```
A small program that helps you manage your dotfiles

USAGE:
    dots [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --store-dir <store-dir>    Directory to use as the dotfile store [default: /Users/jerry/.dotfiles]

SUBCOMMANDS:
    add       Given a path to a file in the home directory, add it to your dotfile store. This will move the
              original file and replace the old location with a symlink into its new location in the dotfile store
    help      Prints this message or the help of the given subcommand(s)
    link      Given a path to a file in the dotfile store, create a symlink to it in the home directory
    list      List the status of all dotfiles in the store
    remove    Given a path to a file in the home directory, remove it from your dotfile store. This will replace the
              symlink in the home directory with the original file from the dotfile store
    unlink    If given a path to a file in the dotfile store, remove the file in the home directory that links to
              it. If given a path to a file in the home directory, remove that file, assuming it links back to a
              file in the dotfile store
```
