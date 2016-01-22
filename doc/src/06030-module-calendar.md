## Calendar {#sec:modules:calendar}

The Calendar module.

It uses the `.ical` files from an external directory and indexes them and puts
entries into the store (for each `.ical` file one).
These entries can be used to reference to a calendar entry.

The module offers the following functionality:

* Adding, removing and setting tags to single entries
* Listing entries, optionally filtered by
  * Tags
  * Match (through the `.ical` file itself)
  * icalendar file entry matches

The calendar module uses an icalendar-library to parse the icalendar files.
The calendar module never writes to the external calendar files.

