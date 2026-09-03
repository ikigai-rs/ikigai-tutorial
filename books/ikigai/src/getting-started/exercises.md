# Exercises

Four ways to make sure the last seven chapters landed. None needs anything beyond
this repository.


1. **Strict lowerCamelCase.** `toCamel` never lower-cases anything. Write `toLowerCamel`
   and decide what it should do with `"XMLHttpRequest"` — there is no obviously right
   answer, which is the point.
2. **A second argument.** Add an optional `separator` so the caller can split on something
   other than whitespace. Declare it `optional()` with a `default_value`, then look at
   `urn:kernel:actions` and watch your own change appear in the catalog.
3. **Take a resource, not a string.** Call your endpoint with `in=urn:fn:toUpper?in=x` and
   satisfy yourself that you did not have to write any code for that to work.
4. **Break cacheability on purpose.** Make an endpoint that returns the current time, mark
   it `.cacheable()`, and observe how convincing a wrong answer looks.

