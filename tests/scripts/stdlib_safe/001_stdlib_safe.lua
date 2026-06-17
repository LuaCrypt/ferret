-- expect: pass
-- category: stdlib_safe

local s = string.char(65 + 1)
print('stdlib_001', s, string.byte(s))
