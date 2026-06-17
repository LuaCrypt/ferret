-- expect: pass
-- category: coroutines

local co = coroutine.create(function()
    coroutine.yield(6)
    return 7
end)
local a, b = coroutine.resume(co)
local c, d = coroutine.resume(co)
print('coroutines_006', a, b, c, d)
