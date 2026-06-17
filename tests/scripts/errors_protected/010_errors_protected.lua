-- expect: pass
-- category: errors_protected

local ok = pcall(function()
    error('err_010')
end)
print('errors_010', ok)
