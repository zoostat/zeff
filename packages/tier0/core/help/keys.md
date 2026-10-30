# keys

## Summary
Field names of a record, or keys of a mapping

## Description
The field names of a record, as text, in the order the fields were written; or the keys of a
mapping in insertion order (section 3.7 of the Language Reference, "Mappings"), with the keys'
own type. `is_in(k, keys(m))` tests whether a mapping has a key before `m[k]`, which is an error
for an absent key (Z0204). Like R's `names`.

## Arguments
- record: A record or a mapping.

## Value
A `List<Str>` for a record; a list of the key type for a mapping.

## See also
is_in, len, unique

## Examples
> keys({mu: 0.0, sd: 1.0})
mu
sd
> let m = [0 => "male", 1 => "female"]
> keys(m)
0
1
> is_in(2, keys(m))
false
