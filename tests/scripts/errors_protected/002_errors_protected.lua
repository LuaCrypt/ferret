-- expect: pass
-- category: errors_protected

local ok = pcall(function()
    error('err_002')
end)
print('errors_002', ok)
