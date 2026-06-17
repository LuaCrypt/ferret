-- expect: pass
-- category: errors_protected

local ok = pcall(function()
    return 7 * 2
end)
print('errors_007', ok)
