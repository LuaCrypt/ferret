-- expect: pass
-- category: errors_protected

local ok = pcall(function()
    return 1 * 2
end)
print('errors_001', ok)
