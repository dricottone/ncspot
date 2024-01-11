# An ncurses Spotify client written in Rust using librespot

ncspot is an ncurses Spotify client written in Rust using librespot.

This is a heavily modified fork.
See the primary project [here](https://github.com/hrkfdn/ncspot).

I have stripped all features that I believe are unnecessary, including:

 + configuration
 + saved state between sessions
 + clipboard interaction
 + graphics (i.e. album art)
 + desktop notifications
 + MPRIS dbus
 + IPC sockets
 + nerdfonts

I have made these additional changes:

 + swapped the preferred backend to ncurses
 + merged a closed PR and un-reverted commit for POSIX signal handling,
   which is incompatible with the upstream project's preferred backend
 + swapped `platform_dirs` for `dirs`
 + refactored various helper functions based on the above removals

