# An ncurses Spotify client written in Rust using librespot

ncspot is an ncurses Spotify client written in Rust using librespot.

This is a heavily modified fork.
See the primary project [here](https://github.com/hrkfdn/ncspot).

I have stripped all features that I believe are unnecessary, including:

 + configuration
 + saved state between sessions
 + clipboard interaction
 + `insert` command (as it's more or less useless *without* a clipboard)
 + graphics (i.e. album art)
 + desktop notifications
 + MPRIS dbus
 + IPC sockets
 + nerdfonts

I have made these additional changes:

 + swapped the preferred backend to `ncurses`
 + merged a closed PR and un-reverted commit for POSIX signal handling,
   which is incompatible with the upstream project's preferred backend
 + swapped `platform_dirs` for `dirs`
 + refactored various helper functions based on the above removals

To what effect?
Taking into account that ~3000 lines deleted are documentation or toolchain...

```
$ git diff origin/main HEAD --stat | tail -n 1
69 files changed, 523 insertions(+), 6340 deletions(-)
```

An approximate net -3000 lines of code.

