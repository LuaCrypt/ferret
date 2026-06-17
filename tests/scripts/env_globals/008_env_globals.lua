-- expect: pass
-- category: env_globals

local outer_print = print
local _ENV = { print = outer_print, value = 80 }
print('env_008', value)
