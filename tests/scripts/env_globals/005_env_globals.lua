-- expect: pass
-- category: env_globals

local outer_print = print
local _ENV = { print = outer_print, value = 50 }
print('env_005', value)
