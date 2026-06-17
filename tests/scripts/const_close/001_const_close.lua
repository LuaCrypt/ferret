-- expect: pass
-- category: const_close

local closed = 0
local guard = setmetatable({}, { __close = function() closed = closed + 1 end })
local x <close> = guard
print('close_001', closed)
