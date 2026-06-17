-- expect: pass
-- category: errors_protected

local ok = pcall(function()
    return 9 * 2
end)
print('errors_009', ok)
