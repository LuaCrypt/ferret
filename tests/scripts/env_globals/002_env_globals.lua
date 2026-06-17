-- expect: pass
-- category: env_globals

local outer_print = print
local _ENV = { print = outer_print, value = 20 }
print('env_002', value)
