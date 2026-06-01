# Ordering

Attributes in comptime code run in order from top to the bottom of a file. Attributes
from sub-modules are run before their parent modules, and attributes from sibling modules
are run in the order their modules are declared in the parent module.

`comptime` blocks within functions are affected by function's arbitrary elaboration order.
Since functions are lazily elaborated now, this will affect ordering of comptime blocks as well.

# Repeat diagnostics

The comptime interpreter can be run on functions which created errors during elaboration.
When this happens, it will execute the function until the error, at which point a repeat
error may be issued. These should be minimized when possible but this needs to be handled
manually in the interpreter by expecting a repeat error variant.

# Mutations and errors

When the comptime interpreter hits an error, it will not revert any mutations it has made
to mutable globals. This can leave these globals in an inconsistent state, potentially leading
to further comptime errors elsewhere.

# Mutating existing items

Comptime functions mutating existing items in the source code should generally be avoided
due to security concerns. These can make auditing code more difficult in particular.
