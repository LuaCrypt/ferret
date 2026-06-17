-- expect: pass
-- category: env_globals

local outer_print = print
local _ENV = { print = outer_print, value = 30 }
print('env_003', value)
