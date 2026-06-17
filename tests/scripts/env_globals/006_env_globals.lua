-- expect: pass
-- category: env_globals

local outer_print = print
local _ENV = { print = outer_print, value = 60 }
print('env_006', value)
