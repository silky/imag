# Modules {#sec:modules}

A module is a functionality of the program.

A module MAY store data in the store (@sec:thestore).
It MAY include user input in the data it stores in the store.
A module MUST HAVE a commandline interface, though a module always consists of
two parts:

- A Library
- A Binary, which
    * is executable by the user
    * implements a commandline frontend to the libray of the module

By definition, the Binary depends on the Library.
By definition, the Library depends on the libstore (@sec:libstore).

A module MUST use the runtime library to implement the commandline
interface as defined in @sec:librt.

The following sections describe the functionality of the modules.

