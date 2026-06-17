-- expect: pass
-- category: stdlib_safe

local s = string.char(65 + 9)
print('stdlib_009', s, string.byte(s))
