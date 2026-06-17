-- expect: pass
-- category: basics

local a = (14 << 2) | 3
local b = (a & 15) ~ 0
print('basics_014', a, b, ~0)
