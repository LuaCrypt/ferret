-- expect: pass
-- category: errors_protected

local ok = pcall(function()
    return 3 * 2
end)
print('errors_003', ok)
