-- expect: pass
-- category: errors_protected

local ok = pcall(function()
    return 5 * 2
end)
print('errors_005', ok)
