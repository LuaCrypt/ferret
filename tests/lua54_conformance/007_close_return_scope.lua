local closed = 0

local function probe()
    local guard = setmetatable({}, { __close = function() closed = closed + 1 end })
    local x <close> = guard
    return closed
end

local before = probe()
print("conf_close_return", before, closed)
