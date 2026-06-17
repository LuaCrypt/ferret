-- expect: pass
-- category: generic_for

local function iter(state, control)
    local next_value = control + 1
    if next_value <= state.limit then
        return next_value, state.base + next_value
    end
end
local total = 0
for key, value in iter, { limit = 5, base = 21 }, 0 do
    total = total + key + value
end
print('generic_007', total)
