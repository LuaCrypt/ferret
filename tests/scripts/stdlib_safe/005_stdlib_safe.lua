-- expect: pass
-- category: stdlib_safe

local s = string.char(65 + 5)
print('stdlib_005', s, string.byte(s))
