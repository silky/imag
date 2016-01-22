## Contacts {#sec:modules:contacts}

The Contacts module uses external `vcard`-files, indexes them and puts
references to them into the store.

These references can be used to link to the contacts.

The module provides functionality to

* add, remove and set tags for each contact
* list contacts, filtered by
  * tags
  * filtering of the `vcard`-files
* printing information about contacts
* printing fields of the `vcard` files with a custom formatter-string, to be
  able to pipe the output into external programs (like `mutt` for example)

The contacts module does not write to the external `vcard` files.

