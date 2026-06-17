local outer_print = print
local _ENV = { print = outer_print, value = 44 }

local function read_value()
    return value
end

print("conf_env_nested", read_value())
