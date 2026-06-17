-- expect: pass
-- category: env_globals

local outer_print = print
local _ENV = { print = outer_print, value = 70 }
print('env_007', value)
