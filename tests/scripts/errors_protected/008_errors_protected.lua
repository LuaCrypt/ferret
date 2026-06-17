-- expect: pass
-- category: errors_protected

local ok = pcall(function()
    error('err_008')
end)
print('errors_008', ok)
