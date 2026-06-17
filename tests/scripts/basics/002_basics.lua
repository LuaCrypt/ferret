-- expect: pass
-- category: basics

local a = (2 << 2) | 3
local b = (a & 15) ~ 2
print('basics_002', a, b, ~0)
