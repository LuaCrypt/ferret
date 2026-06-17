-- expect: pass
-- category: env_globals

local outer_print = print
local _ENV = { print = outer_print, value = 10 }
print('env_001', value)
