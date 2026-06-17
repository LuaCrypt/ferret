-- expect: pass
-- category: env_globals

local outer_print = print
local _ENV = { print = outer_print, value = 40 }
print('env_004', value)
